extern crate diesel;
extern crate time;

use self::diesel::prelude::*;
use super::super::DbConn;
use super::super::models::*;
use super::super::contexts::{ImageList, PasteList, VideoList};
use super::super::contexts::{ManageImage, ManagePaste, ManageVideo};
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
    let mut url = String::from("/manage/request_auth?redirect_path=images/0&auth_secret=");

    if usertoken.is_err() { // They don't have a token yet
        let usertoken = AuthToken::new(_uid);

        let insert_result = diesel::insert(&usertoken)
            .into(schema::auth_tokens::table)
            .get_result::<AuthToken>(&*conn);

        if insert_result.is_err() {
            return Err(Failure(Status::InternalServerError));
        }

        let insert_result = insert_result.unwrap();
        url += insert_result.token.as_str();
    }else {
        // update existing token
        let usertoken = usertoken.unwrap().refresh(); // new token
        // Need identifiable to be implemented
        let update_result = usertoken.save_changes::<AuthToken>(&*conn);

        if update_result.is_err() {
            return Err(Failure(Status::InternalServerError));
        }
        let update_result = update_result.unwrap();

        url += update_result.token.as_str();
    }
    return Ok(status::Custom(Status::Accepted, url));
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
    scookie.set_max_age(time::Duration::days(3));

    cookies.remove_private(Cookie::named("horus_session"));
    cookies.add_private(scookie);
    //cookies.add(Cookie::new("horus_session", token_result.token.clone()));
    Ok(Redirect::to(&redirect_url))
}

#[get("/account")]
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
        Ok(u) => Some(Template::render("manage_account", &u))
    }
}


#[get("/images/<page>")]
pub fn my_images(
    page: u32,
    session: SessionToken,
    conn: DbConn)
    -> Option<Template>
{
    use schema::horus_images::dsl::*;
    use schema::horus_users::dsl::*;
    let images = horus_images
        .filter(owner.eq(&session.uid))
        .order(date_added.desc())
        .limit(24)
        .offset((page * 24) as i64)
        .get_results::<HImage>(&*conn);
    
    let images = images.unwrap();
    let name = horus_users.find(&session.uid)
        .get_result::<User>(&*conn)
        .unwrap().first_name;

    let mut ititle = String::from(name);
    ititle += "'s Images";

    let context = ImageList {
        title: ititle.clone(),
        page_title: ititle,
        editable: false,
        images: images,
    };

    Some(Template::render("manage_images", &context))
}

#[get("/pastes/<page>")]
pub fn my_pastes(
    page: u32,
    session: SessionToken,
    conn: DbConn)
    -> Option<Template>
{
    use schema::horus_pastes::dsl::*;
    use schema::horus_users::dsl::*;

    let pastes = horus_pastes   
        .filter(owner.eq(&session.uid))
        .limit(24)
        .offset((page * 24) as i64)
        .get_results::<HPaste>(&*conn);

    if pastes.is_err() {
        return None;
    }

    let pastes = pastes.unwrap();

    let name = horus_users.find(&session.uid)
        .get_result::<User>(&*conn)
        .unwrap().first_name;
    
    let mut ititle = String::from(name);
    ititle += "'s Pastes";

    let context = PasteList {
        title: ititle.clone(),
        page_title: ititle,
        pastes: pastes,
        editable: false,
    };

    Some(Template::render("manage_pastes", &context))
}

#[get("/videos/<page>")]
pub fn my_videos(
    page: u32,
    session: SessionToken,
    conn: DbConn)
    -> Option<Template>
{
    use schema::horus_videos::dsl::*;
    use schema::horus_users::dsl::*;

    let videos  = horus_videos
        .filter(owner.eq(&session.uid))
        .order(date_added.desc())
        .limit(8)
        .offset((page * 24) as i64)
        .get_results::<HVideo>(&*conn);

    let videos = videos.unwrap();
    let name = horus_users.find(&session.uid)
        .get_result::<User>(&*conn)
        .unwrap().first_name;

    let mut ititle = String::from(name);
    ititle += "'s Videos";
    let context = VideoList {
        title: ititle.clone(),
        page_title: ititle,
        editable: false,
        videos: videos,
    };

    Some(Template::render("manage_videos", &context))
}

#[get("/video/<video_id>")]
pub fn video(
    video_id: String,
    conn: DbConn,
    session: SessionToken)
    -> Option<Template>
{
    use schema::horus_videos::dsl::*;
    let video = horus_videos.find(video_id)
        .get_result::<HVideo>(&*conn);

    if video.is_err() {
        return None;
    }
    let video = video.unwrap();

    if session.uid != video.owner {
        return None;
    }

    let mut ititle = String::new();
    if video.title.is_none() {
        ititle += "Horus Video";
    }else{
        ititle += video.title.unwrap().as_str()
    }
    let context = ManageVideo {
        id: video.id,
        title: ititle.clone(),
        page_title: ititle,
        editable: true,
    };

    Some(Template::render("manage_video", &context))
}

#[get("/image/<image_id>")]
pub fn image(
    image_id: String,
    conn: DbConn,
    session: SessionToken)
    -> Option<Template>
{
    use schema::horus_images::dsl::*;
    let image = horus_images.find(image_id)
        .get_result::<HImage>(&*conn);

    if image.is_err() {
        return None;
    }
    let image = image.unwrap();

    if session.uid != image.owner {
        return None;
    }

    let mut ititle = String::new();
    if image.title.is_none() {
        ititle += "Horus Image";
    }else{
        ititle += image.title.unwrap().as_str()
    }
    let context = ManageImage {
        id: image.id,
        title: ititle.clone(),
        page_title: ititle.clone(),
        editable: true,
    };

    Some(Template::render("manage_image", &context))
}

#[get("/paste/<paste_id>")]
pub fn paste(
    paste_id: String,
    conn: DbConn,
    session: SessionToken)
    -> Option<Template>
{
    use schema::horus_pastes::dsl::*;
    let paste = horus_pastes.find(paste_id)
        .get_result::<HPaste>(&*conn);

    if paste.is_err() {
        return None;
    }
    let paste = paste.unwrap();

    if paste.owner != session.uid {
        return None;
    }

    let mut paste_title = String::from("Horus Paste");
    if paste.title.is_some() {
        paste_title = paste.title.clone().unwrap();
    }

    let context = ManagePaste {
        id: paste.id.clone(),
        title: paste_title.clone(),
        page_title: paste_title,
        paste: paste,
        editable: true,
    };

    Some(Template::render("manage_paste", &context))
}

// REDIRECTS

#[get("/images")]
pub fn my_images_pageless() -> Redirect {
    // Get them to a paged version
    Redirect::to("0")
}

#[get("/videos")]
pub fn my_videos_pageless() -> Redirect {
    // Get them to a paged version
    Redirect::to("0")
}

#[get("/pastes")]
pub fn my_pastes_pageless() -> Redirect {
    // Get them to a paged version
    Redirect::to("0")
}
