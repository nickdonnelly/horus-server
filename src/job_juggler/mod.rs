use std::fmt;
//use std::thread;

use diesel::prelude::*;
use diesel::pg::PgConnection;

use super::schema;
use super::dbtools;
use super::models::{ JobStatus, HJob };

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
            .order(time_queued.desc())
            .limit(4)
            .get_results::<HJob>(&self.connection);
            
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
