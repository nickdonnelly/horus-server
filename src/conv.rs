/// This file holds conversions for various types. Primarily,
/// it is here for request guards.

use chrono::{DateTime, Duration, Local, NaiveDateTime};
use diesel::prelude::*;
use rocket::{Outcome, State};
use rocket::http::Status;
use rocket::request::{self, FromRequest, Request};

use models::{DeploymentKey, HPaste, LicenseKey, SessionToken};
use forms::HNewPasteForm;
use {dbtools, Pool};
use {DbConn, fields::{self, FileName}};

/// Returns a NaiveDateTime given a duration consisting of a string
/// that contains the `type` (`days`, `hours`, or `minutes`) and a value
/// that represents the number of `type`.
pub fn get_dt_from_duration(_type: String, _value: isize) -> Result<NaiveDateTime, String>
{
    if _value <= 0 {
        return Err(format!("Value must be at least 1!"));
    }

    let mut cur_time: DateTime<Local> = Local::now();
    let dur: Option<Duration> = match _type.to_lowercase().as_str() {
        "days" => Some(Duration::days(_value as i64)),
        "hours" => Some(Duration::hours(_value as i64)),
        "minutes" => Some(Duration::minutes(_value as i64)),
        _ => None,
    };

    if dur.is_none() {
        return Err(format!("Unrecognized duration type: {}", _type));
    }

    cur_time = cur_time + dur.unwrap();
    Ok(cur_time.naive_utc())
}

impl<'a, 'r> FromRequest<'a, 'r> for FileName
{
    type Error = String;

    fn from_request(request: &'a Request<'r>) -> request::Outcome<FileName, Self::Error>
    {
        let headers = request.headers();

        if !headers.contains("Content-Disposition") {
            return Outcome::Failure((Status::BadRequest, String::from("Invalid filename")));
        }

        let neutered_filename = String::from(headers.get_one("Content-Disposition").unwrap());
        let neutered_filename = neutered_filename
            .trim()
            .to_lowercase()
            // Yes, I know this is shite. So is working with characters in rust.
            .replace("!", "").replace("*", "")
            .replace("@", "").replace("(", "")
            .replace("#", "").replace(")", "")
            .replace("$", "").replace("+", "")
            .replace("%", "").replace("=", "")
            .replace("^", "").replace("/", "")
            .replace("&", "").replace(",", "")
            .replace(";", "").replace("'", "")
            .replace(":", "").replace("\\", "")
            .replace("|", "").replace("\"", "");

        Outcome::Success(FileName(neutered_filename))
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for DeploymentKey
{
    type Error = String;

    fn from_request(request: &'a Request<'r>) -> request::Outcome<DeploymentKey, Self::Error>
    {
        use schema::deployment_keys::dsl::*;
        use schema;
        use super::routes;

        let keys: Vec<_> = request.headers().get("x-deployment-key").collect();
        let lkey: Vec<_> = request.headers().get("x-api-key").collect();

        if keys.len() != 1 {
            Outcome::Failure((
                Status::Unauthorized,
                "Supply exactly one deployment key.".to_string(),
            ))
        } else {
            let conn = dbtools::get_db_conn(&request).unwrap();
            let key_ = keys[0];
            let lkey = lkey[0];

            // Query database
            let result = deployment_keys.filter(license_key.eq(&lkey)).first(&*conn);

            let lkey = schema::horus_license_keys::dsl::horus_license_keys
                .filter(schema::horus_license_keys::dsl::key.eq(&lkey))
                .first(&*conn);

            if result.is_err() || lkey.is_err() {
                Outcome::Failure((
                    Status::Unauthorized,
                    "Invalid deployment or license key.".to_string(),
                ))
            } else {
                let result: DeploymentKey = result.unwrap();
                let lkey: LicenseKey = lkey.unwrap();

                // Verify hash
                let verification = routes::dist::verify_key(lkey, result, key_.to_string());
                if verification.is_err() {
                    Outcome::Failure((
                        Status::Unauthorized,
                        "Couldn't verify deployment key owner.".to_string(),
                    ))
                } else {
                    Outcome::Success(verification.unwrap())
                }
            }
        }
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for SessionToken
{
    type Error = String;

    fn from_request(request: &'a Request<'r>) -> request::Outcome<SessionToken, Self::Error>
    {
        use schema::session_tokens::dsl::*;

        // TODO: When LocalRequest gets a private_cookie method, undo this being
        // different for tests and just make it always get_private;
        let cookie = request.cookies().get_private("horus_session");

        if cookie.is_none() {
            return Outcome::Forward(());
        }

        let val = String::from(cookie.unwrap().value());

        let conn = dbtools::get_db_conn_requestless();

        if conn.is_err() {
            return Outcome::Forward(());
        }

        let conn = conn.unwrap();

        let stoken = session_tokens
            .filter(token.eq(val))
            .get_result::<SessionToken>(&conn);

        if stoken.is_err() {
            println!("ERROR {:?}", stoken.unwrap());
            return Outcome::Failure((Status::Unauthorized, String::from("Session token invalid.")));
        }
        return Outcome::Success(stoken.unwrap());
    }
}

// LicenseKey
impl<'a, 'r> FromRequest<'a, 'r> for LicenseKey
{
    type Error = String;

    fn from_request(request: &'a Request<'r>) -> request::Outcome<LicenseKey, Self::Error>
    {
        use schema::horus_license_keys::dsl::*;

        // Bad key outcome, here to prevent re-typing

        // Get keys from headers
        let keys: Vec<_> = request.headers().get("x-api-key").collect();
        if keys.len() < 1 {
            return Outcome::Failure((
                Status::Unauthorized,
                String::from("Please provide an API key."),
            ));
        } else if keys.len() > 1 {
            return Outcome::Failure((
                Status::Unauthorized,
                String::from("Please provide only 1 API key."),
            ));
        }

        // Get database handle
        let conn = dbtools::get_db_conn(&request).unwrap();
        let key_ = keys[0];

        // Basic format checks
        if !fields::is_valid_api_key(key_) {
            return Outcome::Failure((
                Status::Unauthorized,
                String::from(format!("Provided key was not valid: {}", key_)),
            ));
        }

        // Query database
        let result = horus_license_keys.find(key_).first(&*conn);

        if result.is_err() {
            return Outcome::Failure((
                Status::Unauthorized,
                String::from(format!("Provided key was not valid: {}", key_)),
            ));
        }

        return Outcome::Success(result.unwrap());
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for DbConn
{
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<DbConn, ()>
    {
        let pool = request.guard::<State<Pool>>().unwrap();
        match pool.get() {
            Ok(conn) => Outcome::Success(DbConn(conn)),
            Err(_) => Outcome::Failure((Status::ServiceUnavailable, ())),
        }
    }
}

impl Into<HPaste> for HNewPasteForm
{
    fn into(self) -> HPaste
    {
        let _id = dbtools::get_random_char_id(8);
        let _date: DateTime<Local> = Local::now();
        HPaste {
            id: _id,
            title: self.title,
            paste_data: self.paste_data,
            owner: -1,
            date_added: _date.naive_utc(),
            is_expiry: self.is_expiry,
            expiration_time: self.expiration_time, // TODO Dont do it this way.
        }
    }
}
