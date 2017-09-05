extern crate r2d2;

use rocket::{State};
use rocket::request::Request;
use super::{DbConn, Pool, fields};
use diesel::pg::PgConnection;
use r2d2_diesel::ConnectionManager;

pub fn get_db_conn(request: &Request) -> Result<DbConn, ()>{
    let pool = request.guard::<State<Pool>>().unwrap();
    let conn = match pool.get() {
        Ok(conn) => Ok(DbConn(conn)),
        Err(_) => Err(()),
    };

    conn
}

// Database pool initialization
pub fn init_pool() -> Pool {
    let config = r2d2::Config::default();
    let manager = ConnectionManager::<PgConnection>::new(super::DATABASE_URL);

    r2d2::Pool::new(config, manager).expect("db pool")
}
