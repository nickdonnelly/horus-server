extern crate chrono;
extern crate diesel;
extern crate regex;

use diesel::prelude::*;
use self::regex::Regex;
use super::models::{AuthToken, DeploymentKey, License, LicenseKey, SessionToken};
use super::routes::manage::AuthRequest;
use super::errors::AuthTokenError;
use super::forms::UserForm;
use super::dbtools;
use self::chrono::Local;
// This file contains the implementations of fields like "email" and other fields that require thorough validation. Only definitions should go here, actual usage should be within new/update methods for any given controller.

// Basically: it's magic.
const EMAIL_REGEX: &str = r"^[^@]+@[^@]+\.[^@]+$";

pub struct FileName(pub String);

pub trait Validatable {
    fn validate_fields(&self) -> Result<(), Vec<String>>;
}

impl Validatable for UserForm {
    fn validate_fields(&self) -> Result<(), Vec<String>> {
        // We handle all errors as a list, this allows us
        // to serialize them to JSON and output them all
        // at once, rather than having only single error
        // messages at any given time.
        let mut errors: Vec<String> = Vec::new();
        if self.first_name.is_some() {
            let fname = self.first_name.clone().unwrap();
            if !is_valid_name_str(fname.as_str()) {
                errors.push(format!("{} is not a valid name", fname.as_str()))
            }
        }

        if self.last_name.is_some() {
            let lname = self.last_name.clone().unwrap();
            if !is_valid_name_str(lname.as_str()) {
                errors.push(format!("{} is not a valid name", lname.as_str()))
            }
        }

        if self.email.is_some() {
            let email_text = self.email.clone().unwrap();
            if !is_valid_email(email_text.as_str()) {
                errors.push(format!("{} is not a valid email", email_text.as_str()))
            }
        }

        if errors.len() > 0 {
            return Err(errors);
        }

        Ok(()) // We don't care about output, we just care
               // about being able to retrieve errors in the event
               // that they exist. So we return a unit.
    }
}

pub fn is_valid_email(email: &str) -> bool {
    let re = Regex::new(EMAIL_REGEX).unwrap();
    return re.is_match(email);
}

// TODO
pub fn is_valid_name_str(name: &str) -> bool {
    !(name.contains(" ") || name.contains(";") || name.contains("_"))
}

/// NOTE: This does not check validity within the database, only checks
/// that the format of the API key is valid!
pub fn is_valid_api_key(key: &str) -> bool {
    if key.len() != 32 || key.contains(" ") {
        return false;
    }
    true
}

impl LicenseKey {
    pub fn belongs_to(&self, uid: i32) -> bool {
        use super::schema::horus_licenses::dsl::*;

        let conn = dbtools::get_db_conn_requestless().unwrap();
        let license = horus_licenses
            .filter(key.eq(&self.key))
            .first::<License>(&conn);

        if license.is_err() {
            return false;
        }

        let license = license.unwrap();

        return license.owner == uid;
    }

    /// This function assumes the LicenseKey object is valid and in the db.
    pub fn get_owner(&self) -> i32 {
        use super::schema::horus_licenses::dsl::*;
        let conn = dbtools::get_db_conn_requestless().unwrap();
        let license = horus_licenses
            .filter(key.eq(&self.key))
            .first::<License>(&conn)
            .unwrap();
        license.owner
    }
}

impl AuthToken {
    pub fn new(uid: i32) -> Self {
        AuthToken {
            uid: uid,
            token: dbtools::get_random_char_id(128),
            expires: None, // use db default
        }
    }

    pub fn refresh(self) -> Self {
        let new_token = dbtools::get_random_char_id(128);

        AuthToken {
            uid: self.uid,
            token: new_token,
            expires: None,
        }
    }
}

impl AuthRequest {
    pub fn into_token(self) -> Result<SessionToken, AuthTokenError> {
        use schema::auth_tokens::dsl::*;
        let conn = dbtools::get_db_conn_requestless();
        if conn.is_err() {
            return Err(AuthTokenError::ConsumeFailure);
        }
        let conn = conn.unwrap();
        let auth_token = auth_tokens
            .filter(token.eq(&self.auth_secret))
            .get_result::<AuthToken>(&conn);

        if auth_token.is_err() {
            return Err(AuthTokenError::NotFound);
        }

        let auth_token = auth_token.unwrap();

        if auth_token.expires.is_none() {
            return Err(AuthTokenError::Invalid);
        }

        let moment = Local::now().naive_utc();
        let etime = auth_token.expires.unwrap();

        if moment > etime {
            //return Err(AuthTokenError::Expired);
        }

        let st = SessionToken::consume_auth_token(auth_token);
        if st.is_none() {
            return Err(AuthTokenError::ConsumeFailure);
        }
        Ok(st.unwrap())
    }
}

impl SessionToken {
    pub fn consume_auth_token(at: AuthToken) -> Option<SessionToken> {
        let stoken = SessionToken {
            uid: at.uid.clone(),
            token: dbtools::get_random_char_id(128),
            use_count: None,
            expires: None,
        };
        let conn = dbtools::get_db_conn_requestless();
        if conn.is_err() {
            return None;
        }
        let conn = conn.unwrap();
        let delete_result = diesel::delete(&at).execute(&conn);

        if delete_result.is_err() {
            return None;
        }

        Some(stoken)
    }
}
