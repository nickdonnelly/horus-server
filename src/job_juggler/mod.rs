use std::fmt;
use std::thread;

use diesel;
use diesel::prelude::*;
use diesel::pg::PgConnection;

use super::schema;
use super::dbtools;
use super::models::{ JobStatus, HJob, NewJob, JobPriority };

pub struct JobJuggler {
    connection: PgConnection,
    job_queue: Vec<HJob>
}

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

    pub fn initialize(&mut self) -> Result<(), JobJugglerError> {
        use schema::horus_jobs::dsl::*;
        // Get a max of 4 pending jobs (some of them have large amounts of data,
        // so we don't want to store too much in ram).
        let start_jobs: Result<Vec<HJob>, _> = horus_jobs
            .filter(job_status.ne(JobStatus::Complete as i32))
            .filter(priority.ne(JobPriority::DoNotProcess as i32))
            .order(priority.desc())
            .order(time_queued.desc())
            .limit(4)
            .get_results::<HJob>(&self.connection);

        // TODO: Mark jobs as queued.
            
        if start_jobs.is_err() {
            return Err(JobJugglerError::new("Couldn't get jobs from database.".to_string()));
        }

        let mut start_jobs = start_jobs.unwrap();
        while !start_jobs.is_empty() {
            let job = start_jobs.pop().unwrap();
            self.job_queue.push(job);
        }

        Ok(())
    }
}

/// Queues a job. Makes a quick query then returns success,
/// then asynchronously uploads the data of the job, so you can
/// use this in a request to make sure the job gets queued without
/// waiting for the data to be sent to the database.
pub fn queue_job(job: NewJob) -> Result<(), JobJugglerError>
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
        let mut db_obj = initial_insert_result.unwrap();
        db_obj.job_data = job.job_data.clone();
        db_obj.priority = job.priority as i32;

        let update_res = diesel::update(horus_jobs.find(db_obj.owner))
            .set(&db_obj)
            .execute(&conn);

        if update_res.is_err() {
            let update_res = diesel::update(horus_jobs.find(db_obj.owner))
                .set(job_status.eq(JobStatus::Failed as i32))
                .execute(&conn);

        }
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
