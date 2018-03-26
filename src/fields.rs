extern crate regex;

use chrono::Local;
use rocket::request::{self, FromRequest};
use rocket::{Outcome, Request};
use diesel::{self, prelude::*};
use self::regex::Regex;

use models::{AuthToken, LicenseKey, SessionToken};
use routes::manage::AuthRequest;
use errors::AuthTokenError;
use forms::UserForm;
use dbtools;

// This file contains the implementations of fields like "email" and other fields that require thorough validation. Only definitions should go here, actual usage should be within new/update methods for any given controller.

// Basically: it's magic.
const EMAIL_REGEX: &str = r"^[^@]+@[^@]+\.[^@]+$";
const ROCKET_ENV: &str = dotenv!("ROCKET_ENV");

pub struct FileName(pub String);

pub trait FromInt
{
    fn from_int(i: i32) -> Self;
}

pub trait Validatable
{
    fn validate_fields(&self) -> Result<(), Vec<String>>;
}

#[derive(FromInt, Clone)]
pub enum PrivilegeLevel
{
    User = 0,
    Admin = 1,
    System = 900,
    God = 901,
}

#[derive(Clone)]
pub enum PrivilegeEnvironment
{
    World,
    Test,
}

pub struct Authentication
{
    priv_lvl: PrivilegeLevel,
    userid: i32,
    environment: PrivilegeEnvironment,
}

impl Authentication
{
    fn new(
        userid: i32,
        privileges: PrivilegeLevel,
        environment: PrivilegeEnvironment,
    ) -> Authentication
    {
        Self {
            priv_lvl: privileges,
            userid: userid,
            environment: environment,
        }
    }

    /// Used for testing purposes. Note that this will panic if the
    /// rocket environment is not dev or development.
    fn new_test_auth(user_id: i32, privilege_level: PrivilegeLevel) -> Authentication
    {
        if ROCKET_ENV != "dev" && ROCKET_ENV != "development" {
            panic!(
                "Attempt to gain test authentication without proper environment! \
                 Environment was {}.",
                ROCKET_ENV
            );
        }

        Authentication::new(user_id, privilege_level, PrivilegeEnvironment::Test)
    }

    /// Get the privilege level associated with the authenticatee
    pub fn get_privilege_level(&self) -> PrivilegeLevel
    {
        self.priv_lvl.clone()
    }

    /// Get the userid of the authenticatee. In the case of `System`,
    /// this should be -1, in the case of `God`, this should be -999.
    pub fn get_userid(&self) -> i32
    {
        self.userid
    }

    /// Only override this if you need a test environment.
    pub fn get_environment(&self) -> PrivilegeEnvironment
    {
        self.environment.clone()
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for Authentication
{
    type Error = String;

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error>
    {
        use rocket::http::Status;

        // Check for a session
        let session_outcome = SessionToken::from_request(request);
        if let Outcome::Success(sesskey) = session_outcome {
            return Outcome::Success(Authentication::new(
                sesskey.uid,
                PrivilegeLevel::from_int(sesskey.privilege_level as i32),
                PrivilegeEnvironment::World,
            ));
        }

        let license_key_outcome = LicenseKey::from_request(request);
        if let Outcome::Success(lkey) = license_key_outcome {
            return Outcome::Success(Authentication::new(
                lkey.get_owner(),
                PrivilegeLevel::from_int(lkey.privilege_level as i32),
                PrivilegeEnvironment::World,
            ));
        }

        // Try test last to reduce overhead in cases with valid auth
        if ROCKET_ENV == "dev" || ROCKET_ENV == "development" {
            if let Some(v) = request.headers().get_one("x-api-test") {
                let s: Vec<&str> = v.split("/").collect();
                if s.len() == 2 {
                    let uid = s[0].parse::<i32>().unwrap();
                    let priv_lvl = PrivilegeLevel::from_int(s[1].parse::<i32>().unwrap());
                    return Outcome::Success(Authentication::new_test_auth(uid, priv_lvl));
                }
            }
        }

        // Not authed.
        return Outcome::Failure((Status::Unauthorized, String::from("Not authorized!")));
    }
}

impl Validatable for UserForm
{
    fn validate_fields(&self) -> Result<(), Vec<String>>
    {
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

pub fn is_valid_email(email: &str) -> bool
{
    let re = Regex::new(EMAIL_REGEX).unwrap();
    return re.is_match(email);
}

// TODO
pub fn is_valid_name_str(name: &str) -> bool
{
    !(name.contains(" ") || name.contains(";") || name.contains("_"))
}

/// NOTE: This does not check validity within the database, only checks
/// that the format of the API key is valid!
pub fn is_valid_api_key(key: &str) -> bool
{
    if key.len() != 32 || key.contains(" ") {
        return false;
    }
    true
}

impl AuthToken
{
    pub fn new(uid: i32, priv_lvl: PrivilegeLevel) -> Self
    {
        AuthToken {
            uid: uid,
            token: dbtools::get_random_char_id(128),
            expires: None, // use db default
            privilege_level: priv_lvl as i32,
        }
    }

    pub fn refresh(self) -> Self
    {
        let new_token = dbtools::get_random_char_id(128);

        AuthToken {
            uid: self.uid,
            token: new_token,
            expires: None,
            privilege_level: self.privilege_level,
        }
    }
}

impl AuthRequest
{
    pub fn into_token(self) -> Result<SessionToken, AuthTokenError>
    {
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

impl SessionToken
{
    pub fn consume_auth_token(at: AuthToken) -> Option<SessionToken>
    {
        let stoken = SessionToken {
            uid: at.uid.clone(),
            token: dbtools::get_random_char_id(128),
            use_count: None,
            expires: None,
            privilege_level: at.privilege_level,
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
