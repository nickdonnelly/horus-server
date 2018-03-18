extern crate ctrlc;

use std::{ process, thread, time, fmt };

use diesel;
use diesel::prelude::*;
use diesel::pg::PgConnection;

use ::schema;
use ::schema::horus_jobs::dsl::*;
use ::dbtools;
use ::models::{ JobStatus, HJob, NewJob, JobPriority };

pub mod job_types;

pub enum JobResult {
    Complete,
    Failed
}

pub struct JobJuggler {
    connection: PgConnection,
    job_queue: Vec<HJob>
}

#[derive(Debug)]
pub struct JobJugglerError {
    desc: String
}

impl JobJuggler {
    
    pub fn new() -> Self
    {
        JobJuggler {
            connection: dbtools::get_db_conn_requestless().unwrap(),
            job_queue: Vec::new()
        }
    }

    pub fn initialize(&mut self) -> Result<(), JobJugglerError> 
    {
        // Get a max of 4 pending jobs (some of them have large amounts of data,
        // so we don't want to store too much in ram).
        let start_jobs: Result<Vec<HJob>, _> = horus_jobs
            .filter(job_status.ne(JobStatus::Complete as i32))
            .filter(priority.ne(JobPriority::DoNotProcess as i32))
            // If, for whatever reason, queued jobs are in there (eg. in event of crash)
            // TODO: This will need to be changed if more than one juggler will run
            // by associating a juggler with each queued job and running a scan if one
            // of the job juggler threads crashes.
            .order(job_status.eq(JobStatus::Queued as i32))
            .order(priority.desc())
            .order(time_queued.asc()) // oldest first
            .limit(4)
            .get_results::<HJob>(&self.connection);
            
        if start_jobs.is_err() {
            return Err(JobJugglerError::new("Couldn't get jobs from database.".to_string()));
        }

        let mut start_jobs = start_jobs.unwrap();

        start_jobs.iter().for_each(|job| {
            let res = diesel::update(horus_jobs.find(job.id))
                .set(job_status.eq(JobStatus::Queued as i32))
                .execute(&self.connection)
                .unwrap();
        });

        while !start_jobs.is_empty() {
            let job = start_jobs.pop().unwrap();
            self.job_queue.push(job);
        }

        Ok(())
    }

    pub fn juggle(self) 
    {
        // Reset status of queued jobs
        ctrlc::set_handler(move || {
            println!("Resetting job status for queued jobs...");
            &self.shutdown();
            process::exit(0);
        }).unwrap();
        // TODO
    }

    /// Resets all jobs to waiting status in the event of SIGTERM
    fn shutdown(&self) 
    {
        self.job_queue.iter().for_each(|job| {
            let res = diesel::update(horus_jobs.find(job.id))
                .set(job_status.eq(JobStatus::Waiting as i32))
                .execute(&self.connection)
                .unwrap();
        });
    }
}

/// Queues a job. Makes a quick query then returns success,
/// then asynchronously uploads the data of the job, so you can
/// use this in a request to make sure the job gets queued without
/// waiting for the data to be sent to the database.
pub fn enqueue_job(job: NewJob) -> Result<(), JobJugglerError>
{
    use schema::horus_jobs::dsl::*;

    let conn = dbtools::get_db_conn_requestless().unwrap();
    let dataless = job.without_data();

    let initial_insert_result = diesel::insert_into(schema::horus_jobs::table)
        .values(&dataless)
        .get_result::<HJob>(&conn);

    if initial_insert_result.is_err() {
        return Err(JobJugglerError::new("Couldn't insert job into database.".to_string()));
    }


    // Spin up separate thread to do the work of moving the data.
    let child = thread::spawn(move || {
        thread::sleep(time::Duration::from_millis(3000));
        let mut db_obj = initial_insert_result.unwrap();
        db_obj.job_data = job.job_data;
        db_obj.priority = job.priority as i32;

        let update_res = diesel::update(horus_jobs.find(db_obj.id))
            .set(&db_obj)
            .execute(&conn);

        if update_res.is_err() {
            eprintln!("Couldn't update job object...Writing failed.");
            let update_res = diesel::update(horus_jobs.find(db_obj.owner))
                .set(job_status.eq(JobStatus::Failed as i32))
                .execute(&conn);
            if update_res.is_err() {
                eprintln!("Couldn't write failure to database! There may be broken jobs!");
            }

        }
        println!("Finished queueing job: id={} => {}", db_obj.id, db_obj.job_name);
    });

    // Our initial insert went fine 
    Ok(())
}

impl fmt::Display for JobJugglerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        write!(f, "{}", self.desc)
    }
}

impl JobJugglerError {
    pub fn new(d: String) -> Self{
        JobJugglerError {
            desc: d
        }
    }
}
