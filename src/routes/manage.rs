extern crate diesel;

use self::diesel::prelude::*;
use super::super::DbConn;
use super::super::models::{LicenseKey, User, HImage, AuthToken, SessionToken};
use super::super::contexts::ImageList;
use super::super::schema;
use super::super::errors::AuthTokenError;
use rocket::response::{status, Failure, Redirect};
use rocket::http::{Cookie, Cookies, Status};
use rocket_contrib::{Template};

#[derive(FromForm)]
pub struct AuthRequest{
    redirect_path: String,
    pub auth_secret: String
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

    let _uid = apikey.get_owner();
    let usertoken = auth_tokens.find(_uid)
        .get_result::<AuthToken>(&*conn);

    if usertoken.is_err() { // They don't have a token yet
        let usertoken = AuthToken::new(_uid);

        let insert_result = diesel::insert(&usertoken)
            .into(schema::auth_tokens::table)
            .get_result::<AuthToken>(&*conn);

        if insert_result.is_err() {
            return Err(Failure(Status::InternalServerError));
        }

        let insert_result = insert_result.unwrap();
        return Ok(status::Custom(Status::Accepted, insert_result.token));

    }else {
        // update existing token
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
#[get("/request_auth?<auth_req>")]
pub fn request_auth_cookie(
    mut cookies: Cookies,
    auth_req: AuthRequest,
    conn: DbConn
    )
    -> Result<Redirect, Failure>
{
    use schema::session_tokens;
    let redirect_url = auth_req.redirect_path.clone();
    
    let token_result = auth_req.into_token();
    if token_result.is_err() {
        return match token_result {
            Err(AuthTokenError::ConsumeFailure) =>
                Err(Failure(Status::InternalServerError)),
            Err(AuthTokenError::Invalid) => Err(Failure(Status::InternalServerError)),
            Err(AuthTokenError::Expired) => Err(Failure(Status::BadRequest)),
            Err(AuthTokenError::NotFound) => Err(Failure(Status::NotFound)),

            // never fires, but has to be here for exhaustiveness..
            Ok(_) => Err(Failure(Status::InternalServerError)), 
        };
    } 
    let token_result = token_result.unwrap();
    let insert_result = token_result.save_changes::<SessionToken>(&*conn);

    //let insert_result = diesel::insert(&token_result)
        //.into(session_tokens::table)
        //.execute(&*conn);

    if insert_result.is_err() {
        println!("ERROR 2! {} ", insert_result.err().unwrap());
        return Err(Failure(Status::InternalServerError));
    }
    //let scookie = Cookie::build("horus_session", token_result.token.clone())
    //    .path("/")
    //    .secure(true)
    //    .finish();

    let mut scookie = Cookie::new("horus_session", token_result.token.clone());
    scookie.set_path("/");

    cookies.remove_private(Cookie::named("horus_session"));
    cookies.add_private(scookie);
    //cookies.add(Cookie::new("horus_session", token_result.token.clone()));
    Ok(Redirect::to(&redirect_url))
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
