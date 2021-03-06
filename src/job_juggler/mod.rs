extern crate ctrlc;

use std::{fmt, process, thread, time};
use std::collections::VecDeque;

use diesel;
use diesel::prelude::*;
use diesel::pg::PgConnection;

use {dbtools, schema};
use models::{HJob, JobPriority, JobStatus, NewJob};
use schema::horus_jobs::dsl::*;

mod job_types;
pub use self::job_types::{ExecutableJob, LoggableJob};

#[derive(Debug)]
pub enum JobResult
{
    Complete,
    Failed,
    FailedWithReason(String)
}

pub struct JobJuggler
{
    connection: PgConnection,
    job_queue: VecDeque<HJob>,
}

impl JobJuggler
{
    pub fn new() -> Self
    {
        JobJuggler {
            connection: dbtools::get_db_conn_requestless().unwrap(),
            job_queue: VecDeque::new(),
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
            return Err(JobJugglerError::new(
                "Couldn't get jobs from database.".to_string(),
            ));
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

    /// The execution loop that completes jobs and queues new ones perpetually.
    pub fn juggle(mut self) -> !
    {
        use std::time::Duration;
        // Reset status of queued jobs
        ctrlc::set_handler(|| {
            println!("Resetting job status for queued jobs...");
            Self::shutdown();
            process::exit(0);
        }).unwrap();

        println!("Starting juggle...");
        loop {
            if !self.job_queue.is_empty() {
                let current_job = self.job_queue.pop_front().unwrap();
                let job_id = current_job.id;
                let current_job_exec = Self::match_job_type(current_job);

                diesel::update(horus_jobs.find(job_id))
                    .set(job_status.eq(JobStatus::Running as i32))
                    .execute(&self.connection)
                    .unwrap();

                let (done_job, result) = current_job_exec.execute(&&self.connection);

                println!("Job finished with result {:?}", result);

                let (result, failure_reason) = Self::match_job_result(result);

                let mut job_logs = done_job.logs();
                if let Some(s) = failure_reason {
                    job_logs.push_str("\n---\nJob failed for reason:\n");
                    job_logs.push_str(&s);
                }

                // Add the result and the logs
                diesel::update(horus_jobs.find(job_id))
                    .set((logs.eq(job_logs), job_status.eq(result)))
                    .execute(&self.connection)
                    .unwrap();
            }

            // Re-fill the queue if it's done
            if self.job_queue.len() < 4 {
                &mut self.check_for_new_job();
            }

            // dont query too often
            thread::sleep(Duration::from_millis(2500));
        }
    }

    /// Enqueues one more job if there is one available
    fn check_for_new_job(&mut self)
    {
        // Get jobs that can be queued by priority, then oldest first
        let new_job: Result<HJob, _> = horus_jobs
            .filter(job_status.eq(JobStatus::Waiting as i32))
            .filter(priority.ne(JobPriority::DoNotProcess as i32))
            .order(priority.desc())
            .order(time_queued.asc()) // oldest first
            .first::<HJob>(&self.connection);

        // Add to queue and set it to queued in the db so it doesn't get multiple queued.
        if new_job.is_ok() {
            let new_job = new_job.unwrap();
            print!("Enqueueing new job (id={})...", new_job.id);
            diesel::update(horus_jobs.find(new_job.id))
                .set(job_status.eq(JobStatus::Queued as i32))
                .execute(&self.connection)
                .unwrap();
            print!("done.\n");

            self.job_queue.push_back(new_job);
        }
    }

    /// Resets all jobs to waiting status in the event of SIGTERM
    fn shutdown()
    {
        let conn = dbtools::get_db_conn_requestless().unwrap();
        diesel::update(horus_jobs.filter(job_status.eq(JobStatus::Queued as i32)))
            .set(job_status.eq(JobStatus::Waiting as i32))
            .execute(&conn)
            .unwrap();
    }

    /// Matches the job string to the correct type of deserialized job.
    fn match_job_type(job: HJob) -> impl ExecutableJob + LoggableJob
    {
        use models::job_structures::*;

        let data = job.job_data.unwrap();
        match job.job_name.as_str() {
            "thumbnail:image" => {
                debinarize::CreateThumbnail(data.as_slice()).unwrap()
            }, // TODO: thumbnail video
            "deployment:deploy:win64" | "deployment:deploy:linux" | _ => {
                debinarize::<Deployment>(data.as_slice()).unwrap()
            },
        }
    }

    /// Returns the job status for the database given the job result.
    /// If it was `JobResult::FailureWithReason`, the failure message is
    /// included in the tuple (otherwise it is `None`).
    fn match_job_result(result: JobResult) -> (i32, Option<String>)
    {
        match result {
            JobResult::Complete => (JobStatus::Complete as i32, None),
            JobResult::Failed => (JobStatus::Failed as i32, None),
            JobResult::FailedWithReason(s) => (JobStatus::Failed as i32, Some(s))
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
        return Err(JobJugglerError::new(
            "Couldn't insert job into database.".to_string(),
        ));
    }

    // Spin up separate thread to do the work of moving the data.
    thread::spawn(move || {
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
        println!(
            "Finished queueing job: id={} => {}",
            db_obj.id, db_obj.job_name
        );
    });

    // Our initial insert went fine
    Ok(())
}


#[derive(Debug)]
pub struct JobJugglerError
{
    desc: String,
}

impl fmt::Display for JobJugglerError
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        write!(f, "{}", self.desc)
    }
}

impl JobJugglerError
{
    pub fn new(d: String) -> Self
    {
        JobJugglerError { desc: d }
    }
}
