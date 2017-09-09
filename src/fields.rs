extern crate regex;

use diesel;
use diesel::expression::Expression;
use self::regex::Regex;
use super::models::*;
// This file contains the implementations of fields like "email" and other fields that require thorough validation. Only definitions should go here, actual usage should be within new/update methods for any given controller.

// Basically: it's magic.
const EMAIL_REGEX: &str = r"^[^@]+@[^@]+\.[^@]+$";

pub trait Validatable {
    fn validate_fields(&self) -> Result<(), Vec<String>>;
}

impl Validatable for UserForm {
    fn validate_fields(&self) -> Result<(), Vec<String>>{
        // We handle all errors as a list, this allows us
        // to serialize them to JSON and output them all
        // at once, rather than having only single error
        // messages at any given time.
        let mut errors: Vec<String> = Vec::new();
        if self.first_name.is_some() {
            let fname = self.first_name.clone().unwrap();
            if !is_valid_name_str(fname.as_str()){
                errors.push(format!("{} is not a valid name", fname.as_str()))
            }
        }

        if self.last_name.is_some() {
            let lname = self.last_name.clone().unwrap();
            if !is_valid_name_str(lname.as_str()){
                errors.push(format!("{} is not a valid name", lname.as_str()))
            }
        }

        if self.email.is_some() {
            let email_text = self.email.clone().unwrap();
           if !is_valid_email(email_text.as_str()){
                errors.push(format!("{} is not a valid email", email_text.as_str()))
            }
        }

        if errors.len() > 0 {
            return Err(errors);
        }

        Ok(())  // We don't care about output, we just care
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
    if key.len() != 32 || key.contains(" "){
        return false;
    }
    true
}

// TODO
pub fn is_valid_paste(_form: &HPasteForm) -> bool {
    true
}
