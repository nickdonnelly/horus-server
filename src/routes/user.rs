use diesel::{ self, prelude::* };
use rocket::response::Failure;
use rocket::response::status;
use rocket::http::Status;
use rocket_contrib::Json;

use ::DbConn;
use ::models::{LicenseKey, User};
use ::forms::UserForm;
use ::schema::horus_users::dsl::*;

// Option usage allows us to automatically 404 if the record is not found
// by just returning "None".
#[get("/<uid>")]
pub fn show(uid: i32, conn: DbConn) -> Option<Json<User>> {
    let user = horus_users.find(uid).first(&*conn);

    if user.is_err() {
        return None;
    }
    Some(Json(user.unwrap()))
}

#[put("/<uid>", format = "application/json", data = "<updated_values>")]
pub fn update(
    uid: i32,
    apikey: LicenseKey,
    updated_values: Json<UserForm>,
    conn: DbConn,
) -> Result<status::Accepted<()>, Failure> {
    if !apikey.belongs_to(uid) {
        return Err(Failure(Status::Unauthorized));
    }

    let user = updated_values.into_inner();

    let result = diesel::update(horus_users.filter(id.eq(uid)))
        .set(&user)
        .execute(&*conn);

    if result.is_err() {
        return Err(Failure(Status::NotFound));
    }

    Ok(status::Accepted(None))
}

#[delete("/<uid>")]
pub fn delete(uid: i32, apikey: LicenseKey, conn: DbConn) -> Result<status::Custom<()>, Failure> {
    if !apikey.belongs_to(uid) {
        return Err(Failure(Status::Unauthorized));
    }

    let result = diesel::delete(horus_users.filter(id.eq(uid))).execute(&*conn);

    if result.is_err() {
        println!(
            "Database error while deleting user: {}",
            result.err().unwrap()
        );
        return Err(Failure(Status::InternalServerError));
    }

    Ok(status::Custom(Status::Ok, ()))
}
