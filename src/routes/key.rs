/// API routes for handling keys.
extern crate chrono;
extern crate rand;
extern crate diesel;
extern crate rocket_contrib;

use diesel::prelude::*;
use super::super::DbConn;
use super::super::schema;
use super::super::dbtools;
use super::super::schema::horus_license_keys::dsl::*;
use super::super::schema::horus_licenses::dsl::*;
use super::super::models::{License, LicenseKey};
use self::chrono::{Local, Date, Duration};
use self::rand::Rng;
use self::rocket_contrib::Json;

// TODO: Temporary - will be secured
#[post("/issue/<uid>")]
pub fn issue(uid: i32) 
    -> Json<(License, LicenseKey)>
{
    Json(issue_license_with_key(uid, 3, 3).unwrap())
}

// TODO: Check if the key is expired before returning
pub fn key_valid(keystr: &str) -> bool
{
    let conn = dbtools::get_db_conn_requestless().unwrap();

    let _key: Result<LicenseKey, diesel::result::Error> = horus_license_keys.filter(
        schema::horus_license_keys::dsl::key.eq(keystr))
        .first(&conn);
    return _key.is_err();
}

/// Endpoint: Check API Key
#[get("/<apikey>/validity-check")]
pub fn validity_check(
    apikey: String, 
    conn: DbConn) 
    -> Result<Json<LicenseKey>, String> 
{
    let _key = horus_license_keys.filter(
        schema::horus_license_keys::dsl::key.eq(&apikey))
        .first(&*conn);

    if _key.is_err() {
        return Err(String::from("invalid"));
    }

    Ok(Json(_key.unwrap()))
}

/// Requestless license issuance. To be used with caution...
pub fn issue_license_with_key(
    owner_id: i32, 
    l_type: i16, 
    priv_lvl: i16) 
    -> Result<(License, LicenseKey), ()> 
{
    // key owner license_type
    let conn = super::super::dbtools::get_db_conn_requestless().unwrap();

    // 32 ASCII characters
    let keystr: String = rand::thread_rng().gen_ascii_chars().take(32).collect();

    let issued: Date<Local> = Local::today();
    let expires_on: Date<Local> = issued + Duration::weeks(104); // 2yr

    let license = License {
        key: keystr.clone(),
        owner: owner_id,
        type_: Some(l_type),
    };
    let l_key = LicenseKey {
        key: keystr.clone(),
        privilege_level: Some(priv_lvl),
        issued_on: issued.naive_utc(),
        valid_until: expires_on.naive_utc(),
    };

    // Insert the records and verify success
    // The key needs to be inserted first due to FK constraints
    let license_key_result = diesel::insert_into(schema::horus_license_keys::table)
        .values(&l_key)
        .get_result::<LicenseKey>(&conn);

    let license_result = diesel::insert_into(schema::horus_licenses::table)
        .values(&license)
        .get_result::<License>(&conn);



    if license_result.is_err() || license_key_result.is_err() {
        // Remove extra records in the event one was successful
        let d_license_result = diesel::delete(
            horus_licenses.filter(schema::horus_licenses::dsl::key.eq(&keystr)))
            .execute(&conn);
        
        let d_license_key_result = diesel::delete(
            horus_license_keys.filter(schema::horus_license_keys::dsl::key.eq(&keystr)))
            .execute(&conn);
    
        if d_license_result.is_err() || d_license_key_result.is_err() {
            println!(
                "NOTICE: Database error while trying to delete faultily created license!");
        }

        return Err(());
    }

    Ok((license_result.unwrap(), license_key_result.unwrap()))
}

