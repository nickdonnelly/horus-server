#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate horus_server;
extern crate rocket;
extern crate rocket_contrib;
#[macro_use] extern crate diesel;
#[macro_use] extern crate diesel_codegen;

use horus_server::*;
use self::models::*;
use self::DbConn;
use diesel::prelude::*;
use rocket_contrib::Template;

use std::collections::HashMap;

#[get("/<name>")]
fn index(name: String, conn: DbConn) -> Template {
    use horus_server::schema::horus_users::dsl::*;

    let mut context = HashMap::new();
    //context.insert("name", name);

    let results = horus_users.filter(first_name.eq(name))
        .load::<User>(&*conn)
        .expect("error");
    
    for user in results {
        context.insert("name", user.email);
        //println!("email {}", user.email);
    }

    Template::render("index", &context)
}

fn main() {
    rocket::ignite()
        .attach(Template::fairing())
        .mount("/name/", routes![index])
        .mount("/user", routes![self::routes::user::show])
        .mount("/user", routes![self::routes::user::update])
        .manage(self::dbtools::init_pool())
        .launch();
}
