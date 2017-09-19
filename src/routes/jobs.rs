use super::super::models::SessionToken;
use super::super::DbConn;
use rocket_contrib::Template;

pub enum JobStatus {
    Queued, // Maps from 0
    Running, // Maps from 1
    Failed, // Maps from 2...etc
    Completed,
}

pub trait PendingJob {
    type Error;
    fn wait_for_completion();
    fn request_cancel() -> bool;
}

pub trait Job {
    type Error;
    type JsonType;

    fn run<T: PendingJob>(self) -> Result<T, Self::Error>;
}


// NONE OF THESE ARE IMPLEMENTED
#[get("/jobs/<uid>")]
pub fn list_jobs(
    uid: u32,
    _session: SessionToken,
    conn: DbConn)
    -> Option<Template>
{
    None
}

#[get("/jobs/<job_id>")]
pub fn job_status(
    job_id: u32,
    _session: SessionToken,
    conn: DbConn)
    -> Option<Template>
{
    None
}
