use std::ops::Deref;

use rocket::State;
use rocket::request::Request;
use diesel::Connection; // Required for trait access to PgConnection
use diesel::pg::PgConnection;
use r2d2_diesel::ConnectionManager;
use {rand, r2d2};
use rand::Rng;

use {DbConn, Pool};

pub mod s3;

/// Retrieves a database connection from the pool given a rocket request
pub fn get_db_conn(request: &Request) -> Result<DbConn, ()>
{
    let pool = request.guard::<State<Pool>>().unwrap();
    let conn = match pool.get() {
        Ok(conn) => Ok(DbConn(conn)),
        Err(_) => Err(()),
    };
    conn
}

/// Retrieves a connection to the database without a request.
/// **Creates a new database connection!** 
/// Excessive use should be avoided as the connection does not come from
/// the pool.
pub fn get_db_conn_requestless() -> Result<PgConnection, ()>
{
    let conn = PgConnection::establish(&super::DATABASE_URL);

    if conn.is_err() {
        return Err(());
    }

    Ok(conn.unwrap())
}


/// Initializes the pool of database connections
pub fn init_pool() -> Pool
{
    let manager = ConnectionManager::<PgConnection>::new(super::DATABASE_URL);

    r2d2::Pool::new(manager).expect("db pool")
}

pub fn get_random_char_id(len: usize) -> String
{
    rand::thread_rng().gen_ascii_chars().take(len).collect()
}

pub fn get_path_image(filename: &str) -> String
{
    let mut path_str = String::from("live/images/");
    path_str += filename;
    path_str += ".png";
    path_str
}

pub fn get_path_file(filename: &str) -> String
{
    let mut path_str = String::from("live/files/");
    path_str += filename;
    path_str
}

pub fn get_path_video(filename: &str) -> String
{
    let mut path_str = String::from("live/videos/");
    path_str += filename;
    path_str += ".webm";
    path_str
}

pub fn get_path_deployment(version: &str, packagename: &str) -> String
{
    let mut path_str = String::from("/live/packages/");
    path_str += version;
    path_str += "/";
    path_str += packagename;
    path_str += ".zip";
    path_str
}

impl Deref for DbConn
{
    type Target = PgConnection;

    fn deref(&self) -> &Self::Target
    {
        &self.0
    }
}
