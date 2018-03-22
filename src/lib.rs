#![feature(custom_derive, plugin, conservative_impl_trait)]
#![plugin(rocket_codegen)]
#![plugin(dotenv_macros)]
#![recursion_limit = "128"] // For diesel schema inference.

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_infer_schema;

#[macro_use]
extern crate horus_server_derive;

extern crate bincode;
extern crate rand;
extern crate serde;
#[allow(unused_extern_crates)]
#[macro_use]
extern crate serde_derive;
#[allow(unused_extern_crates)]
extern crate serde_json;

extern crate chrono;
extern crate r2d2;
extern crate r2d2_diesel;
extern crate rocket;
#[allow(unused_imports)]
#[macro_use]
extern crate rocket_contrib;

use diesel::pg::PgConnection;
use r2d2_diesel::ConnectionManager;

pub mod schema;

pub mod forms; // Modification forms for models
pub mod models;
pub mod dbtools;
pub mod contexts; // Contexts for handlebar templates
pub mod routes;
pub mod fields;
pub mod conv;
pub mod errors;
pub mod job_juggler; // used to manage jobs in the database.

static DATABASE_URL: &'static str = dotenv!("DATABASE_URL");
static AWS_ACCESS: &'static str = dotenv!("AWS_ACCESS");
static AWS_SECRET: &'static str = dotenv!("AWS_SECRET");

// Database pooling definitions
pub type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;
pub struct DbConn(pub r2d2::PooledConnection<ConnectionManager<PgConnection>>);

/// Note, this is NOT a `tests` module but a `test` module. It contains a runner for setup/teardown.
pub mod test {
    use std::panic;

    pub fn run_test<T, U, V>(test: T, setup: U, teardown: V) -> ()
        where T: FnOnce() -> () + panic::UnwindSafe,
              U: FnOnce() -> (),
              V: FnOnce() -> ()

    {
        setup();

        let result = panic::catch_unwind(|| {
            test()
        });

        teardown();

        assert!(result.is_ok());
    }

}
