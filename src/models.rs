#![feature(plugin)]
#![plugin(rocket_codegen)]
extern crate chrono;

use diesel::prelude::*;
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

#[derive(AsChangeset, Deserialize)]
#[table_name="horus_users"]
pub struct UserForm {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
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

#[derive(Queryable, Serialize, Identifiable, Insertable)]
#[table_name="horus_images"]
pub struct HImage {
    pub id: String,
    pub title: Option<String>,
    pub owner: i32,
    pub filepath: String,
    pub date_added: NaiveDate,
    pub is_expiry: bool,
    pub expiration_time: Option<NaiveDateTime>
}

#[derive(Queryable, Serialize, Identifiable, Insertable)]
#[table_name="horus_videos"]
pub struct HVideo {
    pub id: String,
    pub title: Option<String>,
    pub owner: i32,
    pub filepath: String,
    pub date_added: NaiveDate,
    pub is_expiry: bool,
    pub expiration_time: Option<NaiveDateTime>
}

#[derive(Identifiable, Serialize, Insertable, Queryable, Deserialize)]
#[table_name="horus_pastes"]
pub struct HPaste {
    pub id: String,
    pub title: Option<String>,
    pub paste_data: String,
    pub owner: i32,
    pub date_added: NaiveDate,
    pub is_expiry: bool,
    pub expiration_time: Option<NaiveDateTime>
}

#[derive(Deserialize)]
pub struct HPasteForm {
    pub title: Option<String>,
    pub paste_data: String,
    pub owner: i32,
    pub is_expiry: bool,
    pub expiration_time: Option<NaiveDateTime>,
}
