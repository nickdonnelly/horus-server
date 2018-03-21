use ::models::{HJob, LicenseKey, SessionToken, User};
use ::DbConn;

use diesel::prelude::*;
use rocket::response::Failure;
use rocket::http::Status;
use rocket_contrib::{Json, Template};

// NONE OF THESE ARE IMPLEMENTED
#[get("/<_uid>")]
pub fn list_jobs(_uid: u32, _session: SessionToken, _conn: DbConn) -> Option<Template> {
    None
}

#[get("/poll/<job_id>", rank = 1)]
pub fn job_status(job_id: i32, lkey: LicenseKey, conn: DbConn) -> Result<Json<i32>, Failure> {
    let status = poll_job(job_id, lkey.get_owner(), conn);
    match status {
        None => Err(Failure(Status::InternalServerError)),
        Some(v) => Ok(Json(v)),
    }
}

#[get("/poll/<job_id>", rank = 2)]
pub fn job_status_lkey(
    job_id: i32,
    session: SessionToken,
    conn: DbConn,
) -> Result<Json<i32>, Failure> {
    let status = poll_job(job_id, session.uid, conn);
    match status {
        None => Err(Failure(Status::InternalServerError)),
        Some(v) => Ok(Json(v)),
    }
}

/// Poll a job's status. Returns `None` if error.
fn poll_job(job_id: i32, owner_id: i32, conn: DbConn) -> Option<i32> {
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
