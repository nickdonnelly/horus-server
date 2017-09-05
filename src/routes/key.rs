/// API routes for handling keys.
extern crate chrono;

use super::super::DbConn;
use super::super::schema;
use super::super::schema::horus_license_keys::dsl::*;
use super::super::schema::horus_licenses::dsl::*;
use super::super::models::{License, LicenseKey};
use self::chrono::{Local, Date, Duration};
use self::chrono::naive::NaiveDate;

pub fn issue_license(owner_id: i32, l_type: i16, priv_lvl: i16) -> Result<(License, LicenseKey), ()> {
    // key owner license_type
    let conn = super::super::dbtools::get_db_conn().unwrap();
    let key = String::from("");
    let issued: Date<Local> = Local::today();
    let expires_on: Date<Local> = issued + Duration::weeks(104); // 2yr

    let license = License {
        key: key,
        owner: owner_id,
        type_: Some(l_type),
    };
    let lKey = LicenseKey {
        key: key,
        privilege_level: Some(priv_lvl),
        issued_on: issued.naive_utc(),
        valid_until: expires_on.naive_utc(),
    };
    Err(())
}