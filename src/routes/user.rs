#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate diesel;
extern crate rocket;
extern crate rocket_contrib;

use super::super::DbConn;
use super::super::models::{User,UserForm};
use super::super::schema;
use diesel::prelude::*;
use rocket_contrib::{Json, Value};

// Option usage allows us to automatically 404 if the record is not found
// by just returning "None".
#[get("/<uid>")]
pub fn show(uid: i32, conn: DbConn) -> Option<Json<User>> {
    use schema::horus_users::dsl::*;

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
pub fn update(uid: i32, updated_values: Json<UserForm>, conn: DbConn) -> Option<Json<Value>> {
    use schema::horus_users::dsl::*;
    let user = updated_values.into_inner();

    let result = diesel::update(horus_users.filter(id.eq(uid)))
        .set(&user)
        .execute(&*conn);

    if result.is_err() {
        return None;
    }

    Some(Json(json!({ "status":"ok" })))
}