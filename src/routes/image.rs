extern crate rand;
extern crate chrono;

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

#[post("/", format="image/png", data = "<img_data>")]
pub fn new(
    img_data: Data,
    apikey: LicenseKey,
    conn: DbConn)
    -> Result<status::Custom<()>, Failure>
{
    let iid: String = rand::thread_rng().gen_ascii_chars().take(8).collect();

    let mut path_str = String::from("images/");
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

    // TODO Consider using Data.stream_to_file
    Err(Failure(Status::Unauthorized))
}
