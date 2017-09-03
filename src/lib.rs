#![feature(plugin)]
#![plugin(rocket_codegen)]

#[macro_use] extern crate diesel;
#[macro_use] extern crate diesel_codegen;
extern crate serde_json;
#[macro_use] extern crate serde_derive;
extern crate rocket;
#[macro_use] extern crate rocket_contrib;
extern crate dotenv;
extern crate r2d2_diesel;
extern crate r2d2;

use diesel::prelude::*;
use diesel::pg::PgConnection;
use dotenv::dotenv;

use r2d2_diesel::ConnectionManager;

use rocket::request::{self, FromRequest};
use rocket::{Request, State, Outcome};
use rocket::http::Status;

use std::env;
use std::ops::Deref;

pub mod schema;
pub mod models;
pub mod routes;
pub mod fields;
pub mod conv;

static DATABASE_URL: &'static str = env!("DATABASE_URL");

// Database pooling definitions
pub type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;
pub struct DbConn(pub r2d2::PooledConnection<ConnectionManager<PgConnection>>);

impl<'a, 'r> FromRequest<'a, 'r> for DbConn {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<DbConn, ()> {
        let pool = request.guard::<State<Pool>>()?;
        match pool.get() {
            Ok(conn) => Outcome::Success(DbConn(conn)),
            Err(_) => Outcome::Failure((Status::ServiceUnavailable, ()))
        }
    }
}

impl Deref for DbConn {
    type Target = PgConnection;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// Database pool initialization
pub fn init_pool() -> Pool {
    let config = r2d2::Config::default();
    let manager = ConnectionManager::<PgConnection>::new(DATABASE_URL);

    r2d2::Pool::new(config, manager).expect("db pool")
}
