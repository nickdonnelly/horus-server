use rocket::{State, Outcome};
use rocket::http::Status;
use rocket::request::{self, Request, FromRequest};
use super::models::LicenseKey;
use super::Pool;
use super::schema;
use super::{DbConn, fields};
use diesel::prelude::*;

// This only works on valid keys. Will error on invalid ones.
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
        let pool = request.guard::<State<Pool>>().unwrap();
        let conn = match pool.get() {
            Ok(conn) => Ok(DbConn(conn)),
            Err(_) => Err(""),
        };
        let conn = conn.unwrap();

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
