extern crate r2d2;
extern crate rand;

use rocket::{State};
use rocket::request::Request;
use super::{DbConn, Pool};
use diesel::Connection; // Required for trait access to PgConnection
use diesel::pg::PgConnection;
use r2d2_diesel::ConnectionManager;
use dbtools::rand::Rng;

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

pub fn get_random_char_id(len: usize) -> String {
    rand::thread_rng().gen_ascii_chars().take(len).collect()
}

pub fn get_path_image(filename: &str) -> String{
    let mut path_str = String::from("live/images/");
    path_str += filename;
    path_str += ".png";
    path_str
}

pub fn get_path_file(filename: &str, ext: &str) -> String {
    let mut path_str = String::from("live/files/");
    path_str += filename;
    path_str += ".";
    path_str += ext;
    path_str
}

pub fn get_path_video(filename: &str) -> String {
    let mut path_str = String::from("live/videos/");
    path_str += filename;
    path_str += ".webm";
    path_str
}
