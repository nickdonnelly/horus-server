#![feature(plugin)]
#![plugin(rocket_codegen)]
#![recursion_limit="128"] // For diesel schema inference.

#[macro_use] extern crate diesel;
#[macro_use] extern crate diesel_codegen;
extern crate serde_json;
#[macro_use] extern crate serde_derive;
extern crate rocket;
#[macro_use] extern crate rocket_contrib;
extern crate dotenv;
extern crate r2d2_diesel;
extern crate r2d2;

use diesel::pg::PgConnection;
use r2d2_diesel::ConnectionManager;

pub mod schema;
pub mod models;
pub mod routes;
pub mod fields;
pub mod conv;
pub mod dbtools;

static DATABASE_URL: &'static str = env!("DATABASE_URL");

// Database pooling definitions
pub type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;
pub struct DbConn(
    pub r2d2::PooledConnection<ConnectionManager<PgConnection>>);
