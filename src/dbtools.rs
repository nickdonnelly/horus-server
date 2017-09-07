extern crate r2d2;

use rocket::{State};
use rocket::request::Request;
use super::{DbConn, Pool, fields};
use diesel::Connection; // Required for trait access to PgConnection
use diesel::pg::PgConnection;
use r2d2_diesel::ConnectionManager;

pub fn get_db_conn(request: &Request) -> Result<DbConn, ()> {
    let pool = request.guard::<State<Pool>>().unwrap();
    let conn = match pool.get() {
        Ok(conn) => Ok(DbConn(conn)),
        Err(_) => Err(()),
    };
    conn
}

pub fn get_db_conn_requestless() -> Result<PgConnection, ()> {
    let conn = PgConnection::establish(&super::DATABASE_URL);

    if conn.is_err() {
        return Err(());
    }

    Ok(conn.unwrap())
}

// Database pool initialization
pub fn init_pool() -> Pool {
    let config = r2d2::Config::default();
    let manager = ConnectionManager::<PgConnection>::new(super::DATABASE_URL);

    r2d2::Pool::new(config, manager).expect("db pool")
}
