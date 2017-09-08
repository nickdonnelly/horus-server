/// This file holds conversions for various types. Primarily,
/// it is here for request guards.

use rocket::{State, Outcome};
use rocket::http::Status;
use rocket::request::{self, Request, FromRequest};
use super::models::LicenseKey;
use super::Pool;
use super::schema;
use super::dbtools;
use super::{DbConn, fields};
use diesel::prelude::*;
use std::ops::Deref;


// LicenseKey
impl<'a, 'r> FromRequest<'a, 'r> for LicenseKey {
    type Error = String;

    fn from_request(request: &'a Request<'r>) -> request::Outcome<LicenseKey, Self::Error> {
        use schema::horus_license_keys::dsl::*;
        
        // Bad key outcome, here to prevent re-typing

        // Get keys from headers
        let keys: Vec<_> = request.headers().get("x-api-key").collect();
        if keys.len() < 1 {
            return Outcome::Failure((Status::BadRequest, String::from("Please provide an API key.")));
        }else if keys.len() > 1{
            return Outcome::Failure((Status::BadRequest, String::from("Please provide only 1 API key.")));
        }

        // Get database handle
        let conn = dbtools::get_db_conn(&request).unwrap();
        let key_ = keys[0];

        // Basic format checks
        if !fields::is_valid_api_key(key_) {
            return Outcome::Failure((Status::Unauthorized, String::from(format!("Provided key was not valid: {}", key_))));

        }

        // Query database
        let result = horus_license_keys.find(key_)
            .first(&*conn);

        if result.is_err() {
            return Outcome::Failure((Status::Unauthorized, String::from(format!("Provided key was not valid: {}", key_))));
        }

        return Outcome::Success(result.unwrap()); 
    }
}

// DbConn
impl<'a, 'r> FromRequest<'a, 'r> for DbConn {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<DbConn, ()> {
        let pool = request.guard::<State<Pool>>().unwrap();
        match pool.get() {
            Ok(conn) => Outcome::Success(DbConn(conn)),
            Err(_) => Outcome::Failure((Status::ServiceUnavailable, ())),
        }
    }
}


impl Deref for DbConn {
    type Target = PgConnection;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
