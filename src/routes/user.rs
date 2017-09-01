#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;
extern crate rocket_contrib;

use super::super::DbConn;
use super::super::models::User;
use super::super::schema;
use diesel::prelude::*;
use rocket_contrib::Json;

#[get("/<uid>")]
pub fn show(uid: i32, conn: DbConn) -> Json<User> {
    use schema::horus_users::dsl::*;

    let user = horus_users.filter(id.eq(1))
        .first(&*conn)
        .expect("error");
    // let user = horus_users.find(id)
    //     .first(&*conn)
    //     //.load::<User>(&*conn)
    //     .expect("error");
    
        Json(user)
}