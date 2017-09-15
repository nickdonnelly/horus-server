extern crate diesel;

use self::diesel::prelude::*;
use super::super::DbConn;
use super::super::models::{LicenseKey, User, HImage, AuthToken};
use super::super::contexts::ImageList;
use super::super::schema;
use rocket::response::{status, Failure, Redirect};
use rocket::http::Status;
use rocket_contrib::{Template};

#[derive(FromForm)]
pub struct AuthRequest{
    redirect_path: String,
    auth_secret: String
}


/// Gives dtapp a URL to open in the users browser that will auth
/// them with a cookie then redirect them. Consider adding additional
/// request guards to this...
/// Overwrites any current auth tokens for the user making the request.
#[post("/request_auth_url")]
pub fn request_auth_url(
    apikey: LicenseKey,
    conn: DbConn)
    -> Result<status::Custom<String>, Failure> 
{
    use schema::auth_tokens::dsl::*;

    let uid = apikey.get_owner();
    let usertoken = auth_tokens.find(uid)
        .get_result::<AuthToken>(&*conn);
    if usertoken.is_err() {
        let usertoken = AuthToken::new(uid);

        let insert_result = diesel::insert(&usertoken)
            .into(schema::auth_tokens::table)
            .get_result::<AuthToken>(&*conn);

        if insert_result.is_err() {
            return Err(Failure(Status::InternalServerError));
        }

        let insert_result = insert_result.unwrap();
        return Ok(status::Custom(Status::Accepted, insert_result.token));

    }else {
        // update token
        let usertoken = usertoken.unwrap().refresh(); // new token
        // Need identifiable to be implemented
        let update_result = usertoken.save_changes::<AuthToken>(&*conn);

        if update_result.is_err() {
            return Err(Failure(Status::InternalServerError));
        }
        let update_result = update_result.unwrap();

        return Ok(status::Custom(Status::Accepted, update_result.token));
    }
}

/// Stores auth cookie + redirects user to redirect_path
/// This should be a SESSION token not an AUTH
#[post("/request_auth?<auth_req>")]
pub fn request_auth_cookie(
    auth_req: AuthRequest,
    apikey: LicenseKey,
    conn: DbConn
    )
    -> Result<Redirect, Failure>
{
    let valid_result = !auth_req.is_valid(&conn);
    if valid_result.is_err() {
        return Err(Failure(Status::Unauthorized));
    }
    let valid_result = valid_result.unwrap();

    Err(Failure(Status::InternalServerError))
}

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
