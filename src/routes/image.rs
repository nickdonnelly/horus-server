extern crate diesel; // this might not even be necessary but im not deleting it
extern crate rocket;
extern crate rocket_contrib;

use self::diesel::prelude::*;
use super::super::DbConn;
use super::super::fields;
use self::rocket::response::Failure;
use self::rocket::response::status;
use self::rocket::http::Status;
use super::super::models::{HPaste, HPasteForm, LicenseKey};

#[get("/<id>")]
pub fn show(
    paste_id: &'a str, 
    conn: DbConn) 
    -> Option<String> 
{
    use super::super::schema::horus_pastes::dsl::*;
    let paste = horus_pastes.find(paste_id)
        .load::<HPaste>(&*conn);

    if paste.is_err() {
        return None;
    }

    Some(paste.paste_data);
}

/// Acceptance string is the ID of the new paste
#[post("/new", format = "", data = "<paste>")]
pub fn new(
    paste: Json<HPasteForm>, 
    _apikey: LicenseKey,
    conn: DbConn)
    -> Result<status::Accepted<String>, Failure>
{
    let paste_form_data = paste.to_inner();
    if !fields::is_valid_paste(&paste_form_data) {
        return Failure(Status::BadRequest);
    }


}
