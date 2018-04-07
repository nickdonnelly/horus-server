use diesel::prelude::*;
use rocket::response::Failure;
use rocket::http::Status;
use rocket_contrib::Json;

use models::{HJob, JobStatus, User};
use fields::{Authentication, PrivilegeLevel};
use schema::horus_jobs::dsl::*;
use schema::horus_users::dsl::*;
use DbConn;

#[derive(Serialize)]
pub struct ListJob {id: i32, job_name: String, job_status: i32, priority: i32}

// NONE OF THESE ARE IMPLEMENTED
#[get("/active/<uid>")]
pub fn list_jobs(uid: i32, auth: Authentication, conn: DbConn) 
    -> Result<Json<Vec<ListJob>>, Failure>
{
    if auth.get_userid() != uid && auth.get_privilege_level() == PrivilegeLevel::User {
        return Err(Failure(Status::Unauthorized));
    }

    let user = horus_users.find(&uid).get_result::<User>(&*conn);

    if let Err(_) = user {
        return Err(Failure(Status::NotFound));
    }

    let user = user.unwrap();

    let result = HJob::belonging_to(&user)
       .filter(job_status.ne(JobStatus::Failed as i32))
       .filter(job_status.ne(JobStatus::Complete as i32))
       .select((::schema::horus_jobs::dsl::id, job_name, job_status, priority))
       .get_results::<(i32, String, i32, i32)>(&*conn);

    match result {
        Ok(values) => {
            let values = values.iter().map(|&(id, ref name, status, _priority)| {
                ListJob {
                    id: id, job_name: name.clone(), job_status: status, priority: _priority
                }
            }).collect();

            Ok(Json(values))
        },
        Err(_) => Err(Failure(Status::InternalServerError))
    }
}


#[get("/poll/<job_id>")]
pub fn retrieve_job_status(job_id: i32, auth: Authentication, conn: DbConn) 
    -> Result<Json<i32>, Failure>
{
    let status = poll_job(job_id, auth.get_userid(), conn);
    match status {
        None => Err(Failure(Status::NotFound)),
        Some(v) => Ok(Json(v)),
    }
}

/// Poll a job's status. Returns `None` if error.
fn poll_job(job_id: i32, owner_id: i32, conn: DbConn) -> Option<i32>
{
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
