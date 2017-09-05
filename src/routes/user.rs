extern crate diesel;
extern crate rocket;
extern crate rocket_contrib;

use super::super::DbConn;
use diesel::prelude::*;
use super::super::models::{User,UserForm,LicenseKey};
use super::super::schema;
use super::super::schema::horus_users::dsl::*;
use rocket_contrib::{Json, Value};

// Option usage allows us to automatically 404 if the record is not found
// by just returning "None".
#[get("/<uid>")]
pub fn show(uid: i32, conn: DbConn) -> Option<Json<User>> {

    // let user = horus_users.filter(id.eq(uid))
        // .first(&*conn);
    let user = horus_users.find(uid)
        .first(&*conn);
    //     //.load::<User>(&*conn)
    //     .expect("error
    if user.is_err() {
        return None;
    }
    Some(Json(user.unwrap()))
}

#[put("/<uid>", format = "application/json", data = "<updated_values>")]
pub fn update(uid: i32, _apikey: LicenseKey, updated_values: Json<UserForm>, conn: DbConn) -> Option<Json<Value>> {
    let user = updated_values.into_inner();

    let result = diesel::update(horus_users.filter(id.eq(uid)))
        .set(&user)
        .execute(&*conn);

    if result.is_err() {
        return None;
    }

    Some(Json(json!({ "status":"ok" })))
}

//#[delete("/<uid>"]
//pub fn delete(uid: i32, apikey: LicenseKey, conn: DbConn) -> Option<Json<Value>> {
    // TODO: Check if license matches user id before deleting.

//}
