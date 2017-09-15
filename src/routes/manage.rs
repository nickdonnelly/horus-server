extern crate diesel;

use self::diesel::prelude::*;
use super::super::DbConn;
use super::super::models::{LicenseKey, User, HImage, HVideo};
use super::super::contexts::ImageList;
use rocket::response::{status, Failure, Redirect};
use rocket::http::Status;
use rocket_contrib::{Template, Json};

#[get("/")]
pub fn my_account(
    apikey: LicenseKey,
    conn: DbConn)
    -> Option<Template>
{
    use schema::horus_users::dsl::*;
    let uid = apikey.get_owner();

    let user = horus_users.filter(id.eq(&uid))
        .first::<User>(&*conn);

    match user {
        Err(_) => None,
        Ok(u) => Some(Template::render("myaccount", &u))
    }
}


// TODO
#[get("/images/<page>")]
pub fn my_images(
    page: u32,
    apikey: LicenseKey,
    conn: DbConn)
    -> Option<Template>
{
    use schema::horus_images::dsl::*;
    let images = horus_images
        .filter(owner.eq(apikey.get_owner()))
        .order(date_added.desc())
        .limit(24)
        .offset((page * 24) as i64)
        .get_results::<HImage>(&*conn);
    
    let images = images.unwrap();

    let context = ImageList {
        first_name: String::from("TODO"),
        images: images,
    };

    Some(Template::render("images", &context))
}

// REDIRECTS

#[get("/images")]
pub fn my_images_pageless() -> Redirect {
    // Get them to a paged version
    Redirect::to("/images/1")
}
#[get("/videos")]
pub fn my_videos_pageless() -> Redirect {
    // Get them to a paged version
    Redirect::to("/videos/1")
}

#[get("/pastes")]
pub fn my_pastes_pageless() -> Redirect {
    // Get them to a paged version
    Redirect::to("/pastes/1")
}
