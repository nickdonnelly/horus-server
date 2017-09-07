#![feature(plugin)]
#![plugin(rocket_codegen)]
#![allow(deprecated)]

/// API routes for handling keys.
extern crate chrono;
extern crate rand;
extern crate diesel;
extern crate rocket;
extern crate rocket_contrib;

use diesel::prelude::*;
use super::super::DbConn;
use super::super::schema;
use super::super::schema::horus_license_keys::dsl::*;
use super::super::schema::horus_licenses::dsl::*;
use super::super::models::{License, LicenseKey};
use self::chrono::{Local, Date, Duration};
use self::chrono::naive::NaiveDate;
use self::rand::Rng;
use self::rocket_contrib::Json;

// TODO: Temporary - will be secured
#[post("/issue/<uid>")]
pub fn issue(uid: i32) 
    -> Json<(License, LicenseKey)>
{
    Json(issue_license_with_key(1, 3, 3).unwrap())
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
        return Err(String::from(""));
    }

    Ok(Json(_key.unwrap()))
}

/// Requestless license issuance.
/// TODO: Transfer this to a single relation, it's overly verbose at the moment
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
    let license_key_result = diesel::insert(&l_key)
        .into(schema::horus_license_keys::table)
        .get_result::<LicenseKey>(&conn);

    let license_result = diesel::insert(&license)
        .into(schema::horus_licenses::table)
        .get_result::<License>(&conn);



    if license_result.is_err() || license_key_result.is_err() {
        // Remove extra records in the event one was successful
        diesel::delete(
            horus_licenses.filter(schema::horus_licenses::dsl::key.eq(&keystr)))
            .execute(&conn);
        
        diesel::delete(
            horus_license_keys.filter(schema::horus_license_keys::dsl::key.eq(&keystr)))
            .execute(&conn);


        return Err(());
    }

    Ok((license_result.unwrap(), license_key_result.unwrap()))
}

