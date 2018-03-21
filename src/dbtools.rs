extern crate s3;

use std::process;
use std::ops::Deref;

use self::s3::bucket::Bucket;
use self::s3::credentials::Credentials;
use rocket::State;
use rocket::request::Request;
use diesel::Connection; // Required for trait access to PgConnection
use diesel::pg::PgConnection;
use r2d2_diesel::ConnectionManager;
use {r2d2, rand};
use rand::Rng;

use ::{DbConn, Pool};

const BUCKET: &'static str = "horuscdn";
const REGION: &'static str = "eu-central-1";

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

pub fn delete_s3_object(path: &str) -> Result<String, ()> {
    let creds = get_s3_creds();
    let region = REGION.parse::<self::s3::region::Region>().unwrap();
    let bucket = Bucket::new(BUCKET, region, creds);
    let res = bucket.delete(path);
    if res.is_err() {
        return Err(());
    }

    let (data, _) = res.unwrap();

    Ok(String::from_utf8(data).unwrap())
}

/// Upload a public, named resource to s3.
/// returns: a public URL to the object
pub fn resource_to_s3_named(filename: &str, path: &str, data: &Vec<u8>) -> Result<String, ()> {
    let creds = get_s3_creds();
    let region = REGION.parse::<self::s3::region::Region>().unwrap();
    let mut bucket = Bucket::new(BUCKET, region, creds);
    let mut dispositionstr = String::from("attachment; filename=\"");
    dispositionstr += filename;
    dispositionstr += "\"";
    bucket.add_header("x-amz-acl", "public-read"); // this way we can serve it later
    bucket.add_header("content-disposition", &dispositionstr);

    let (by, code) = bucket.put(&path, &data, "text/plain").unwrap();

    if code != 200 {
        return Err(());
    }
    Ok(String::from_utf8(by).unwrap())
}

/// Upload to S3 using the private canned ACL.
/// returns: the path to the s3 object from the root of the bucket (not a url)
pub fn private_resource_to_s3_named(
    filename: &str,
    path: &str,
    data: &Vec<u8>,
) -> Result<String, ()> {
    let creds = get_s3_creds();
    let region = REGION.parse::<self::s3::region::Region>().unwrap();
    let mut bucket = Bucket::new(BUCKET, region, creds);
    let mut dispositionstr = String::from("attachment; filename=\"");
    dispositionstr += filename;
    dispositionstr += "\"";
    bucket.add_header("x-amz-acl", "private"); // only we can read
    bucket.add_header("content-disposition", &dispositionstr);
    let (_, code) = bucket
        .put(&path, &data, "application/octet-stream")
        .unwrap();

    if code != 200 {
        Err(())
    } else {
        Ok(String::from(path))
    }
}

pub fn resource_to_s3(path: &str, data: &Vec<u8>) -> Result<String, ()> {
    let creds = Credentials::new(&super::AWS_ACCESS, &super::AWS_SECRET, None);
    let region = REGION.parse::<self::s3::region::Region>().unwrap();
    let mut bucket = Bucket::new(BUCKET, region, creds);
    bucket.add_header("x-amz-acl", "public-read"); // this way we can serve it later
    bucket.add_header("content-disposition", "attachment");

    let (by, code) = bucket.put(&path, &data, "text/plain").unwrap();

    if code != 200 {
        return Err(());
    }
    Ok(String::from_utf8(by).unwrap())
}

// Database pool initialization
pub fn init_pool() -> Pool {
    //let config = r2d2::Config::default();
    let manager = ConnectionManager::<PgConnection>::new(super::DATABASE_URL);

    r2d2::Pool::new(manager).expect("db pool")
}

pub fn get_random_char_id(len: usize) -> String {
    rand::thread_rng().gen_ascii_chars().take(len).collect()
}

pub fn get_path_image(filename: &str) -> String {
    let mut path_str = String::from("live/images/");
    path_str += filename;
    path_str += ".png";
    path_str
}

pub fn get_path_file(filename: &str) -> String {
    let mut path_str = String::from("live/files/");
    path_str += filename;
    path_str
}

pub fn get_path_video(filename: &str) -> String {
    let mut path_str = String::from("live/videos/");
    path_str += filename;
    path_str += ".webm";
    path_str
}

pub fn get_path_deployment(version: &str, packagename: &str) -> String {
    let mut path_str = String::from("/live/packages/");
    path_str += version;
    path_str += "/";
    path_str += packagename;
    path_str += ".zip";
    path_str
}

/// Return a pre-signed URL, for a path starting at the root of the crate.
pub fn get_s3_presigned_url(path: String) -> Result<String, String> {
    use self::process::Command;
    
    let mut url_base = "s3://".to_string() + BUCKET;
    url_base += path.as_str(); 

    let url = Command::new("aws")
        .arg("s3")
        .arg("presign")
        .arg("--expires-in")
        .arg("60") // seconds.
        .arg(url_base)
        .output();

    if url.is_err() {
        eprintln!("Couldn't get a presigned download URL: {}", url.err().unwrap());
        Err("Couldn't get a presigned download URL.".to_string())
    } else {
        let url = url.unwrap();
        Ok(String::from_utf8_lossy(&url.stdout).to_string())
    }
}

fn get_s3_creds() -> Credentials {
    Credentials::new(&super::AWS_ACCESS, &super::AWS_SECRET, None)
}

impl Deref for DbConn {
    type Target = PgConnection;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
