extern crate chrono;
extern crate base64;
extern crate diesel;

use diesel::prelude::*;
use super::super::DbConn;
use super::super::dbtools;
use super::super::models::{LicenseKey,HImage};
use super::super::forms::HImageChangesetForm;
use rocket::response::{Failure, NamedFile};
use rocket::response::status;
use rocket::data::Data;
use rocket::http::Status;
use rocket_contrib::{Json, Template};
use self::chrono::Local;

use std::collections::HashMap;
use std::path::Path;
use std::io::Read;
use std::io::prelude::*;
use std::fs::File;

#[get("/<image_id>")]
pub fn show(
    image_id: String,
    conn: DbConn)
    -> Option<Template>
{
    use schema::horus_images::dsl::*;

    let image = horus_images.find(&image_id)
        .get_result::<HImage>(&*conn);

    if image.is_err() {
        return None;
    }
    let image = image.unwrap();
    Some(Template::render("show_image", &image))
}

#[get("/full/<image_id>")]
pub fn full(
    image_id: String,
    conn: DbConn)
    -> Option<NamedFile>
{
    use schema::horus_images::dsl::*;
    let image = horus_images.find(image_id)
        .get_result::<HImage>(&*conn);

    if image.is_err() {
        return None;
    }
    let image = image.unwrap();

    let image_path = Path::new(&image.filepath);
    NamedFile::open(image_path).ok()
}

#[get("/thumb/<image_id>")]
pub fn thumb(
    image_id: String,
    conn: DbConn)
    -> Option<NamedFile>
{
    full(image_id, conn)
}

/// `list` returns a paginated JSON array of HImage objects.
/// Pages start at index 0.
#[get("/<uid>/list/<page>")]
pub fn list(
    uid: i32,
    page: u32,
    apikey: LicenseKey,
    conn: DbConn)
    -> Result<Json<Vec<HImage>>, Failure>
{
    use schema::horus_images::dsl::*;

    if !apikey.belongs_to(uid) {
        return Err(Failure(Status::Unauthorized));
    }

    let images = horus_images
        .filter(owner.eq(uid))
        .order(date_added.desc())
        .limit(24)
        .offset((page * 24) as i64)
        .get_results::<HImage>(&*conn);

    if images.is_err() {
        return Err(Failure(Status::InternalServerError));
    }

    Ok(Json(images.unwrap()))
}

#[delete("/<image_id>")]
pub fn delete(
    image_id: String,
    apikey: LicenseKey,
    conn: DbConn)
    -> Result<status::Custom<()>, Failure>
{
    use schema::horus_images::dsl::*;

    let image = horus_images
        .filter(id.eq(&image_id))
        .first::<HImage>(&*conn);

    if image.is_err() {
        return Err(Failure(Status::NotFound));
    }

    let image = image.unwrap();
    
    if !apikey.belongs_to(image.owner) {
        return Err(Failure(Status::Unauthorized));
    }

    let result = diesel::delete(&image).execute(&*conn);

    if result.is_err() {
        println!("Database error while deleting image: {}", result.err().unwrap());
        return Err(Failure(Status::InternalServerError));
    }

    Ok(status::Custom(Status::Ok, ()))
}


/// Note: This doesn't support custom titles yet.
/// Also TODO: Abstract the fuck out of this
#[post("/", format="image/png", data = "<img_data>")]
pub fn new(
    img_data: Data,
    apikey: LicenseKey,
    conn: DbConn)
    -> Result<status::Created<()>, Failure>
{
    use schema::horus_images;
    let iid: String = dbtools::get_random_char_id(8);

    let pathstr = dbtools::get_path_image(&iid);

    let image = HImage {
        id: iid.clone(),
        title: None,
        owner: apikey.get_owner(),
        filepath: pathstr.clone(),
        date_added: Local::now().naive_utc(),
        is_expiry: false,
        expiration_time: None,
    };
    // SAVE THE FILE THEN INSERT DB
    let img_data: Vec<u8> = img_data.open()
        .bytes()
        .map(|x| x.unwrap())
        .collect();

    // Removes the prefix
    let raw_img_data = base64::decode(&img_data[20..]);

    if raw_img_data.is_err() {
        return Err(Failure(Status::BadRequest));
    }

    let raw_img_data = raw_img_data.unwrap();

    let path: &Path = Path::new(&pathstr);
    let buffer = File::create(&path);

    if buffer.is_err() {
        return Err(Failure(Status::BadRequest));
    }

    let mut buffer = buffer.unwrap();

    let buffer = buffer.write(&raw_img_data);
    
    let result = diesel::insert(&image)
        .into(horus_images::table)
        .get_result::<HImage>(&*conn);
    
    if buffer.is_err() || result.is_err() {
        return Err(Failure(Status::InternalServerError));
    }
    
    let result = result.unwrap();

    // TODO Consider using Data.stream_to_file
    Ok(status::Created(String::from("/image/") + result.id.as_str(), None))
}

#[put("/<image_id>", format = "application/json", data = "<updated_values>")]
pub fn update(
    image_id: String,
    updated_values: Json<HImageChangesetForm>,
    apikey: LicenseKey,
    conn: DbConn)
    -> Result<status::Accepted<()>, Failure>
{
    use schema::horus_images::dsl::*;

    let img = horus_images.filter(id.eq(&image_id))
        .first::<HImage>(&*conn);

    if img.is_err() {
        return Err(Failure(Status::NotFound));
    }
    let img = img.unwrap();

    if !apikey.belongs_to(img.owner) {
        return Err(Failure(Status::Unauthorized));
    }

    let img_update = updated_values.into_inner();
    let result = diesel::update(horus_images.filter(id.eq(image_id)))
        .set(&img_update)
        .execute(&*conn);

    match result {
        Ok(_) => Ok(status::Accepted(None)),
        Err(_) => Err(Failure(Status::InternalServerError)),
    }
}

