extern crate ctrlc;

use std::{ process, thread, time, fmt };
use std::collections::VecDeque;

use diesel;
use diesel::prelude::*;
use diesel::pg::PgConnection;

use ::{ DbConn, dbtools, schema };
use ::schema::horus_jobs::dsl::*;
use ::models::{ JobStatus, HJob, NewJob, JobPriority };

mod job_types;
pub use self::job_types::{ LoggableJob, ExecutableJob };

pub enum JobResult {
    Complete,
    Failed
}

pub struct JobJuggler {
    connection: PgConnection,
    job_queue: VecDeque<HJob>
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
            job_queue: VecDeque::new()
        }
    }

    pub fn initialize(&mut self) -> Result<(), JobJugglerError> 
    {
        // Gets a max of 4 pending jobs (some of them have large amounts of data,
        // so we don't want to store too much in ram) and puts them into the queue.
        // TODO: Automatically mark jobs that are "running" as failed - again see
        // comment below on if we plan to have more than one juggler running concurrently.
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
            diesel::update(horus_jobs.find(job.id))
                .set(job_status.eq(JobStatus::Queued as i32))
                .execute(&self.connection)
                .unwrap();
        });

        while !start_jobs.is_empty() {
            let job = start_jobs.remove(0);
            self.job_queue.push_back(job);
        }

        Ok(())
    }

    pub fn juggle(mut self) -> ! 
    {
        use std::time::Duration;
        // Reset status of queued jobs
        ctrlc::set_handler(|| {
            println!("Resetting job status for queued jobs...");
            Self::shutdown();
            process::exit(0);
        }).unwrap();

        loop {
            if !self.job_queue.is_empty() {
                let current_job = self.job_queue.pop_front().unwrap();
                let job_id = current_job.id;
                let current_job_exec = Self::match_job_type(current_job);

                
                diesel::update(horus_jobs.find(job_id))
                    .set(job_status.eq(JobStatus::Running as i32))
                    .execute(&self.connection)
                    .unwrap();

                let result = match current_job_exec.execute(&&self.connection) {
                    JobResult::Complete => JobStatus::Complete,
                    JobResult::Failed => JobStatus::Failed
                } as i32;

                diesel::update(horus_jobs.find(job_id))
                    .set(job_status.eq(result))
                    .execute(&self.connection)
                    .unwrap();

                // dont query too often
                thread::sleep(Duration::from_millis(2500)); 
                &mut self.check_for_new_job();
            }
        }
    }

    fn check_for_new_job(&mut self)
    {
        let new_job: Result<HJob, _> = horus_jobs
            .filter(job_status.ne(JobStatus::Complete as i32))
            .filter(priority.ne(JobPriority::DoNotProcess as i32))
            .order(priority.desc())
            .order(time_queued.asc()) // oldest first
            .first::<HJob>(&self.connection);

        if new_job.is_ok() {
            self.job_queue.push_back(new_job.unwrap());
        }
    }

    /// Resets all jobs to waiting status in the event of SIGTERM
    fn shutdown() 
    {
        let conn = dbtools::get_db_conn_requestless().unwrap();
        diesel::update(
            horus_jobs.filter(job_status.eq(JobStatus::Queued as i32)))
            .set(job_status.eq(JobStatus::Waiting as i32))
            .execute(&conn)
            .unwrap();
    }

    fn match_job_type(job: HJob)  -> impl ExecutableJob
    {
        use ::models::job_structures::*;

        match job.job_name.as_str() {
            "deployment:deploy:win64"|"deployment:deploy:linux"|_ 
                => debinarize::<Deployment>(job.job_data.unwrap().as_slice())
        }
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
