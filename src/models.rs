extern crate chrono;

use self::chrono::NaiveDate;
use self::chrono::NaiveDateTime;
use super::schema::*;

#[derive(Queryable, Identifiable, Serialize, Deserialize)]
#[table_name="horus_users"]
pub struct User {
    pub id: i32,
    pub first_name: String,
    pub last_name: Option<String>,
    pub email: String,
}

#[derive(Identifiable, Insertable, Queryable, AsChangeset)]
#[primary_key(uid)]
#[table_name="auth_tokens"]
pub struct AuthToken {
    pub uid: i32,
    pub token: String,
    pub expires: Option<NaiveDateTime>,
}

#[derive(Debug)]
#[derive(Identifiable, Insertable, Queryable, AsChangeset)]
#[primary_key(uid)]
#[table_name="session_tokens"]
pub struct SessionToken {
    pub uid: i32,
    pub token: String,
    pub use_count: Option<i32>,
    pub expires: Option<NaiveDateTime>,
}

#[derive(Insertable, Queryable, Serialize, Deserialize, AsChangeset)]
#[table_name="deployment_keys"]
pub struct DeploymentKey {
    key: String,
    pub deployments: i32,
    pub license_key: String
}

#[derive(Insertable, Queryable, Serialize)]
#[table_name="horus_license_keys"]
pub struct LicenseKey {
    pub key: String,
    pub privilege_level: Option<i16>,
    pub issued_on: NaiveDate, // DO NOT MEASURE TIME
    pub valid_until: NaiveDate, // WITH THESE VALUES! NOT MONOTONIC!
    //pub rate_limit: u32,
}

#[derive(Insertable, Queryable, Serialize)]
#[table_name="horus_licenses"]
pub struct License {
    pub key: String,
    pub owner: i32,
    pub type_: Option<i16>, // This way we still match "type", which is 
                            // otherwise a rust-reserved keyword.
}

#[derive(AsChangeset, Queryable, Serialize, Identifiable, Insertable)]
#[table_name="horus_images"]
pub struct HImage {
    pub id: String,
    pub title: Option<String>,
    pub owner: i32,
    pub filepath: String,
    pub date_added: NaiveDateTime,
    pub is_expiry: bool,
    pub expiration_time: Option<NaiveDateTime>
}

#[derive(Queryable, Serialize, Identifiable, Insertable, AsChangeset)]
#[table_name="horus_files"]
pub struct HFile {
    pub id: String,
    pub owner: i32,
    pub filename: String,
    pub filepath: String,
    pub date_added: NaiveDateTime,
    pub is_expiry: bool,
    pub expiration_time: Option<NaiveDateTime>,
    pub download_counter: Option<i32>,
}

#[derive(AsChangeset, Queryable, Serialize, Identifiable, Insertable)]
#[table_name="horus_videos"]
pub struct HVideo {
    pub id: String,
    pub title: Option<String>,
    pub owner: i32,
    pub filepath: String,
    pub date_added: NaiveDateTime,
    pub is_expiry: bool,
    pub expiration_time: Option<NaiveDateTime>
}

#[derive(AsChangeset, Identifiable, Serialize, Insertable, Queryable, Deserialize)]
#[table_name="horus_pastes"]
pub struct HPaste {
    pub id: String,
    pub title: Option<String>,
    pub paste_data: String,
    pub owner: i32,
    pub date_added: NaiveDateTime,
    pub is_expiry: bool,
    pub expiration_time: Option<NaiveDateTime>
}

#[derive(Identifiable, Queryable)]
#[table_name="horus_jobs"]
pub struct HJob {
    pub id: i32,
    pub owner: i32,
    pub job_status: super::routes::jobs::JobStatus,
    pub job_data: String,
    time_queued: Option<NaiveDateTime>,
}

impl DeploymentKey {
    pub fn new(key_hash: String, lkey: &LicenseKey) -> Self {
        Self {
            key: key_hash,
            license_key: lkey.key.clone(),
            deployments: 0
        }
    }
}
