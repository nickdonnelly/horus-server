extern crate diesel; // this might not even be necessary but im not deleting it

use self::diesel::prelude::*;
use super::super::DbConn;
use rocket::response::Failure;
use rocket::response::status;
use rocket::http::Status;
use rocket_contrib::Json;
use schema::horus_pastes::dsl::*;
use super::super::models::{HPaste, LicenseKey, SessionToken};
use super::super::forms::{HNewPasteForm, HPasteChangesetForm};
use rocket_contrib::Template;
use std::collections::HashMap;

#[get("/<paste_id>")]
pub fn show(
    paste_id: String, 
    conn: DbConn) 
    -> Option<Template> 
{
    let paste = horus_pastes.find(paste_id)
        .first::<HPaste>(&*conn);

    if paste.is_err() {
        return None;
    }
    let paste = paste.unwrap();

    let mut context = HashMap::new();
    context.insert("paste_data", paste.paste_data);
    if paste.title != None {
        context.insert("title", paste.title.unwrap());
    }else{
        context.insert("title", "Horus Paste".to_string());
    }

    Some(Template::render("show_paste", &context))
}

#[get("/<uid>/list/<page>")]
pub fn list(
    uid: i32,
    page: u32,
    apikey: LicenseKey,
    conn: DbConn)
    -> Result<Json<Vec<HPaste>>, Failure>
{
    if !apikey.belongs_to(uid) {
        return Err(Failure(Status::Unauthorized));
    }

    let pastes = horus_pastes
        .filter(owner.eq(uid))
        .order(date_added.desc())
        .limit(48)
        .offset((page * 48) as i64)
        .get_results::<HPaste>(&*conn);

    if pastes.is_err() {
        println!("Paste selection failed with error: {}", pastes.err().unwrap());
        return Err(Failure(Status::InternalServerError));
    }

    Ok(Json(pastes.unwrap()))
}

/// Acceptance string is the ID of the new paste
#[post("/new", format = "application/json", data = "<paste>")]
pub fn new(
    paste: Json<HNewPasteForm>, 
    apikey: LicenseKey,
    conn: DbConn)
    -> Result<status::Created<()>, Failure>
{
    use schema::horus_pastes;

    let paste_form_data = paste.into_inner();
    let mut paste: HPaste = paste_form_data.into();
    paste.owner = apikey.get_owner();

    let result = diesel::insert(&paste)
        .into(horus_pastes::table)
        .get_result::<HPaste>(&*conn);

    if result.is_err() {
        return Err(Failure(Status::InternalServerError));
    }

    let result = result.unwrap();

    Ok(status::Created(String::from("/paste/") + result.id.as_str(), None))
}

fn delete_internal(
    paste: HPaste,
    conn: DbConn)
    -> Result<status::Custom<()>, Failure>
{
    let result = diesel::delete(&paste).execute(&*conn);

    if result.is_err() {
        println!("Databse error while deleting paste: {}", result.err().unwrap());
        return Err(Failure(Status::InternalServerError));
    }
    
    Ok(status::Custom(Status::Ok, ()))
}

#[delete("/<paste_id>")]
pub fn delete(
    paste_id: String,
    session: SessionToken,
    conn: DbConn)
    -> Result<status::Custom<()>, Failure>
{
    let paste = horus_pastes
        .find(paste_id)
        .get_result::<HPaste>(&*conn);

    if paste.is_err() {
        return Err(Failure(Status::NotFound))
    }

    let paste = paste.unwrap();

    if session.uid != paste.owner {
        return Err(Failure(Status::Unauthorized))
    }
    
    delete_internal(paste, conn)
}

#[delete("/<paste_id>", rank = 2 )]
pub fn delete_sessionless(
    paste_id: String,
    apikey: LicenseKey,
    conn: DbConn)
    -> Result<status::Custom<()>, Failure>
{
    let paste = horus_pastes
        .find(paste_id)
        .get_result::<HPaste>(&*conn);

    if paste.is_err() {
        return Err(Failure(Status::NotFound))
    }

    let paste = paste.unwrap();

    if !apikey.belongs_to(paste.owner) {
        return Err(Failure(Status::Unauthorized))
    }

    delete_internal(paste, conn)
}

#[put("/<paste_id>", format = "application/json", data = "<updated_values>")]
pub fn update(
    paste_id: String,
    updated_values: Json<HPasteChangesetForm>,
    session: SessionToken,
    conn: DbConn)
    -> Result<status::Accepted<()>, Failure>
{
    let paste = horus_pastes.filter(id.eq(&paste_id))
        .first::<HPaste>(&*conn);

    if paste.is_err() {
        return Err(Failure(Status::NotFound));
    }
    let paste = paste.unwrap();

    if session.uid != paste.owner {
        return Err(Failure(Status::Unauthorized));
    }

    let paste_update = updated_values.into_inner();
    let result = diesel::update(horus_pastes.filter(id.eq(paste_id)))
        .set(&paste_update)
        .execute(&*conn);

    match result {
        Ok(_) => Ok(status::Accepted(None)),
        Err(_) => Err(Failure(Status::InternalServerError)),
    }
}
