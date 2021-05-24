use diesel::{self, prelude::*};
use rocket::response::status;
use rocket::http::Status;
use rocket_contrib::json::Json;

use DbConn;
use models::{LicenseKey, PublicUser, User};
use fields::Authentication;
use forms::UserForm;
use schema::horus_users::dsl::*;

// Option usage allows us to automatically 404 if the record is not found
// by just returning "None".
#[get("/<uid>")]
pub fn show(uid: i32, conn: DbConn) -> Option<Json<PublicUser>>
{
    let user = horus_users.find(uid).first::<User>(&*conn);

    if user.is_err() {
        return None;
    }

    Some(Json(user.unwrap().without_sensitive_attributes()))
}

#[get("/me")]
pub fn show_privileged(auth: Authentication, conn: DbConn) -> Option<Json<User>>
{
    let user = horus_users.find(auth.get_userid()).first::<User>(&*conn);

    if user.is_err() {
        return None;
    }

    Some(Json(user.unwrap()))
}

#[put("/<uid>", format = "application/json", data = "<updated_values>")]
pub fn update(
    uid: i32,
    auth: Authentication,
    updated_values: Json<UserForm>,
    conn: DbConn,
) -> Result<status::Accepted<()>, status::Custom<()>>
{
    if auth.get_userid() != uid {
        return Err(status::Custom(Status::Unauthorized, ()));
    }

    let user = updated_values.into_inner();

    let result = diesel::update(horus_users.filter(id.eq(uid)))
        .set(&user)
        .execute(&*conn);

    if result.is_err() {
        return Err(status::Custom(Status::NotFound, ()));
    }

    Ok(status::Accepted(None))
}

// NOTE: Keep this is apikey: have the user enter it in a modal to delete their account.
#[delete("/<uid>")]
pub fn delete(uid: i32, apikey: LicenseKey, conn: DbConn) -> Result<status::Custom<()>, status::Custom<()>>
{
    if !apikey.belongs_to(uid) {
        return Err(status::Custom(Status::Unauthorized, ()));
    }

    let result = diesel::delete(horus_users.filter(id.eq(uid))).execute(&*conn);

    if result.is_err() {
        println!(
            "Database error while deleting user: {}",
            result.err().unwrap()
        );
        return Err(status::Custom(Status::InternalServerError, ()));
    }

    Ok(status::Custom(Status::Ok, ()))
}
