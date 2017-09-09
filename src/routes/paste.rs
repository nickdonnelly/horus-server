extern crate diesel; // this might not even be necessary but im not deleting it
extern crate rocket;
extern crate rocket_contrib;

use self::diesel::prelude::*;
use super::super::DbConn;
use super::super::fields;
use super::super::schema;
use self::rocket::response::Failure;
use self::rocket::response::status;
use self::rocket::http::Status;
use self::rocket_contrib::Json;
use schema::horus_pastes::dsl::*;
use super::super::models::{HPaste, HPasteForm, LicenseKey};

#[get("/<paste_id>")]
pub fn show(
    paste_id: String, 
    conn: DbConn) 
    -> Option<String> 
{
    let paste = horus_pastes.find(paste_id)
        .first::<HPaste>(&*conn);

    if paste.is_err() {
        return None;
    }

    Some(paste.unwrap().paste_data)
}

/// Acceptance string is the ID of the new paste
#[post("/new", format = "application/json", data = "<paste>")]
pub fn new(
    paste: Json<HPasteForm>, 
    _apikey: LicenseKey,
    conn: DbConn)
    -> Result<status::Created<()>, Failure>
{
    use schema::horus_pastes;

    let paste_form_data = paste.into_inner();
    if !fields::is_valid_paste(&paste_form_data) {
        return Err(Failure(Status::BadRequest));
    }

    let paste: HPaste = paste_form_data.into();

    let result = diesel::insert(&paste)
        .into(horus_pastes::table)
        .get_result::<HPaste>(&*conn);

    if result.is_err() {
        return Err(Failure(Status::InternalServerError));
    }

    let result = result.unwrap();

    Ok(status::Created(String::from("/paste/") + result.id.as_str(), None))
}
