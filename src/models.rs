extern crate chrono;

use diesel::associations::Identifiable;
use self::chrono::NaiveDate;
use super::schema::*;

#[derive(Queryable, Identifiable, Serialize, Deserialize)]
#[table_name="horus_users"]
pub struct User {
    pub id: i32,
    pub first_name: String,
    pub last_name: Option<String>,
    pub email: String,
}

#[derive(AsChangeset, Deserialize)]
#[table_name="horus_users"]
pub struct UserForm {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
}

#[derive(Queryable)]
pub struct LicenseKey {
    pub license_key: String,
    pub privilege_level: Option<i16>,
    pub issued_on: NaiveDate, // DO NOT MEASURE TIME
    pub valid_until: NaiveDate, // WITH THESE VALUES! NOT MONOTONIC!
    //pub rate_limit: u32,
}

#[derive(Queryable)]
pub struct License {
    pub key: String,
    pub owner: u32,
    pub license_type: i16,
}
