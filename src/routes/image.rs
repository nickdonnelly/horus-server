extern crate rand;
extern crate chrono;
extern crate base64;
extern crate diesel;

use diesel::prelude::*;
use super::super::DbConn;
use super::super::schema;
use super::super::models::{LicenseKey,HImage};
use rocket::response::Failure;
use rocket::response::status;
use rocket::data::Data;
use rocket::http::Status;
use rocket_contrib::Template;
use self::chrono::{Local, Date};
use self::rand::Rng;

use std::collections::HashMap;
use std::path::Path;
use std::io::{Error, Read};
use std::io::prelude::*;
use std::fs::File;

#[get("/<image_id>")]
pub fn show(
    image_id: String,
    conn: DbConn)
    -> Option<Template>
{
    let mut context = HashMap::new();
    context.insert("image_url", "test");
    Some(Template::render("image", &context))
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
    let iid: String = rand::thread_rng().gen_ascii_chars().take(8).collect();

    let mut path_str = String::from("live/images/");
    path_str += iid.as_str();
    path_str += ".png";

    let path: &Path = Path::new(&path_str);

    let image = HImage {
        id: iid,
        title: None,
        owner: apikey.get_owner(),
        filepath: path_str.clone(),
        date_added: Local::today().naive_utc(),
        is_expiry: false,
        expiration_time: None,
    };
    // SAVE THE FILE THEN INSERT DB
    let img_data: Vec<u8> = img_data.open()
        .bytes()
        .map(|x| {
            // TODO: Only unwrap if this is a byte that comes after
            // the base64 MIME prefix.
            x.unwrap()
        }).collect();

    let raw_img_data = base64::decode(&img_data[20..]);

    if raw_img_data.is_err() {
        return Err(Failure(Status::BadRequest));
    }

    let raw_img_data = raw_img_data.unwrap();
    let buffer = File::create(&path);

    if buffer.is_err() {
        return Err(Failure(Status::BadRequest));
    }

    let mut buffer = buffer.unwrap();

    buffer.write(&raw_img_data);
    
    let result = diesel::insert(&image)
        .into(horus_images::table)
        .get_result::<HImage>(&*conn);
    
    if result.is_err() {
        return Err(Failure(Status::InternalServerError));
    }
    let result = result.unwrap();

    // TODO Consider using Data.stream_to_file
    Ok(status::Created(String::from("/image/") + result.id.as_str(), None))
}

