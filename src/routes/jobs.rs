use super::super::models::{ HJob, JobStatus, SessionToken, User };
use super::super::DbConn;
use super::super::schema;

use diesel::prelude::*;
use rocket::response::{ status, Failure };
use rocket::http::Status;
use rocket_contrib::{ Json, Template };


// NONE OF THESE ARE IMPLEMENTED
#[get("/<_uid>")]
pub fn list_jobs(
    _uid: u32,
    _session: SessionToken,
    _conn: DbConn)
    -> Option<Template>
{
    None
}

#[get("/poll/<job_id>", rank=1)]
pub fn job_status(
    job_id: i32,
    session: SessionToken,
    _conn: DbConn)
    -> Result<Json<JobStatus>, Failure>
{
    Err(Failure(Status::InternalServerError))    
}

#[get("/poll/<job_id>", rank=2)]
pub fn job_status_lkey(
    job_id: i32,
    session: SessionToken,
    _conn: DbConn)
    -> Result<Json<JobStatus>, Failure>
{
    Err(Failure(Status::InternalServerError))    
}

/// Poll a job's status. Returns `None` if error.
fn poll_job(job_id: i32, owner_id: i32, conn: DbConn) -> Option<i32>
{
     use schema::horus_jobs::dsl::*;
     use schema::horus_users::dsl::*;

     let user = horus_users.find(owner_id).first::<User>(&*conn);

     if user.is_err() {
        return None;
     }
    
     let user = user.unwrap();
     let result = HJob::belonging_to(&user)
        .find(job_id)
        .select(job_status)
        .first::<i32>(&*conn);

     if result.is_err() {
        None
     } else {
        Some(result.unwrap())
     }
}
