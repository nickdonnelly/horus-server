extern crate time;

use diesel::{self, prelude::*};
use rocket::response::{status, Failure, Redirect};
use rocket::http::{Cookie, Cookies, Status};
use rocket_contrib::templates::Template;

use DbConn;
use models::{AuthToken, HFile, HImage, HPaste, HVideo, LicenseKey, SessionToken, User};
use fields::Authentication;
use contexts::{FileList, ImageList, PasteList, VideoList};
use contexts::{ManageImage, ManagePaste, ManageVideo, ManageFile};
use contexts::ShowAccount;
use schema;
use errors::AuthTokenError;

#[derive(FromForm)]
pub struct AuthRequest
{
    redirect_path: String,
    pub auth_secret: String,
}

/// Gives dtapp a URL to open in the users browser that will auth
/// them with a cookie then redirect them. Consider adding additional
/// request guards to this...
/// Overwrites any current auth tokens for the user making the request.
#[get("/request_auth_url")]
pub fn request_auth_url(apikey: LicenseKey, conn: DbConn)
    -> Result<status::Custom<String>, Failure>
{
    use schema::auth_tokens::dsl::*;
    use fields::PrivilegeLevel;
    use from_int::FromInt;

    let _uid = apikey.get_owner();
    let usertoken = auth_tokens.find(_uid).get_result::<AuthToken>(&*conn);
    let mut url = String::from("/manage/request_auth?redirect_path=images/0&auth_secret=");

    if usertoken.is_err() {
        // They don't have a token yet
        let usertoken = AuthToken::new(
            _uid,
            PrivilegeLevel::from_int(apikey.privilege_level as i32).unwrap(),
        );

        let insert_result = diesel::insert_into(schema::auth_tokens::table)
            .values(&usertoken)
            .get_result::<AuthToken>(&*conn);

        if insert_result.is_err() {
            return Err(Failure(Status::InternalServerError));
        }

        let insert_result = insert_result.unwrap();
        url += insert_result.token.as_str();
    } else {
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
    conn: DbConn,
) -> Result<Redirect, Failure>
{
    let redirect_url = auth_req.redirect_path.clone();

    let token_result = auth_req.into_token();
    if token_result.is_err() {
        return match token_result {
            Err(AuthTokenError::ConsumeFailure) => Err(Failure(Status::InternalServerError)),
            Err(AuthTokenError::Invalid) => Err(Failure(Status::InternalServerError)),
            Err(AuthTokenError::Expired) => Err(Failure(Status::BadRequest)),
            Err(AuthTokenError::NotFound) => Err(Failure(Status::NotFound)),

            // never fires, but has to be here for exhaustiveness..
            Ok(_) => Err(Failure(Status::InternalServerError)),
        };
    }
    let token_result = token_result.unwrap();
    let insert_result = token_result.save_changes::<SessionToken>(&*conn);

    if insert_result.is_err() {
        let insert_new_result = diesel::insert_into(schema::session_tokens::table)
            .values(&token_result)
            .execute(&*conn);

        if insert_new_result.is_err() {
            return Err(Failure(Status::InternalServerError));
        }
    }

    let mut scookie = Cookie::new("horus_session", token_result.token.clone());
    scookie.set_path("/");
    scookie.set_max_age(time::Duration::days(3));

    cookies.remove_private(Cookie::named("horus_session"));
    cookies.add_private(scookie);
    Ok(Redirect::to(&redirect_url))
}

#[get("/account")]
pub fn my_account(auth: Authentication, conn: DbConn) -> Option<Template>
{
    use schema::horus_users::dsl::*;
    use schema::horus_licenses::dsl::*;
    use diesel::dsl::sum;

    let uid = auth.get_userid();

    let user = horus_users.filter(id.eq(&uid)).first::<User>(&*conn);

    if user.is_err() {
        return None;
    }

    let user = user.unwrap();
    let resource_counts = horus_licenses.filter(owner.eq(user.id))
        .select(sum(resource_count))
        .first::<Option<i64>>(&*conn)
        .unwrap();


    let context = ShowAccount {
        user_id: user.id,
        first_name: user.first_name,
        last_name: user.last_name,
        email: user.email,
        privilege_level: auth.get_privilege_level().to_string(),
        resource_count: resource_counts.unwrap()
    };

    Some(Template::render("manage_account", &context))
}

#[get("/images/<page>")]
pub fn my_images(page: u32, auth: Authentication, conn: DbConn) -> Option<Template>
{
    use schema::horus_images::dsl::*;
    use schema::horus_users::dsl::*;
    let images = horus_images
        .filter(owner.eq(auth.get_userid()))
        .order(date_added.desc())
        .limit(24)
        .offset((page * 24) as i64)
        .get_results::<HImage>(&*conn);

    let images = images.unwrap().iter().map(move |img|{
        img.with_displayable_date()
    }).collect();

    let name = horus_users
        .find(auth.get_userid())
        .get_result::<User>(&*conn)
        .unwrap()
        .first_name;

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

#[get("/files/<page>")]
pub fn my_files(page: u32, auth: Authentication, conn: DbConn) -> Option<Template>
{
    use schema::horus_files::dsl::*;
    use schema::horus_users::dsl::*;

    let files = horus_files
        .filter(owner.eq(auth.get_userid()))
        .limit(24)
        .order(date_added.desc())
        .offset((page * 24) as i64)
        .get_results::<HFile>(&*conn);

    if files.is_err() {
        return None;
    }

    let files = files.unwrap();

    let name = horus_users
        .find(auth.get_userid())
        .get_result::<User>(&*conn)
        .unwrap()
        .first_name;

    let mut ititle = String::from(name);
    ititle += "'s Files";

    let context = FileList {
        title: ititle.clone(),
        page_title: ititle,
        files: files,
        editable: false,
    };

    Some(Template::render("manage_files", &context))
}

#[get("/pastes/<page>")]
pub fn my_pastes(page: u32, auth: Authentication, conn: DbConn) -> Option<Template>
{
    use schema::horus_pastes::dsl::*;
    use schema::horus_users::dsl::*;

    let pastes = horus_pastes
        .filter(owner.eq(auth.get_userid()))
        .limit(24)
        .order(date_added.desc())
        .offset((page * 24) as i64)
        .get_results::<HPaste>(&*conn);

    if pastes.is_err() {
        return None;
    }

    let pastes = pastes.unwrap();

    let name = horus_users
        .find(auth.get_userid())
        .get_result::<User>(&*conn)
        .unwrap()
        .first_name;

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
pub fn my_videos(page: u32, auth: Authentication, conn: DbConn) -> Option<Template>
{
    use schema::horus_videos::dsl::*;
    use schema::horus_users::dsl::*;

    let videos = horus_videos
        .filter(owner.eq(auth.get_userid()))
        .order(date_added.desc())
        .limit(8)
        .offset((page * 24) as i64)
        .get_results::<HVideo>(&*conn);

    let videos = videos.unwrap();
    let name = horus_users
        .find(auth.get_userid())
        .get_result::<User>(&*conn)
        .unwrap()
        .first_name;

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
pub fn video(video_id: String, conn: DbConn, auth: Authentication) -> Option<Template>
{
    use schema::horus_videos::dsl::*;
    let video = horus_videos.find(video_id).get_result::<HVideo>(&*conn);

    if video.is_err() {
        return None;
    }
    let video = video.unwrap();

    if auth.get_userid() != video.owner {
        return None;
    }

    let mut ititle = String::new();
    if video.title.is_none() {
        ititle += "Horus Video";
    } else {
        ititle += video.title.unwrap().as_str()
    }

    let path = if video.password == None {
        None
    } else {
        Some(::dbtools::s3::get_s3_presigned_url(video.filepath).unwrap())
    };

    let context = ManageVideo {
        id: video.id,
        title: ititle.clone(),
        page_title: ititle,
        is_expiry: video.is_expiry,
        date_added: format!("{}", video.date_added),
        editable: true,
        password: video.password,
        vid_src: path
    };

    Some(Template::render("manage_video", &context))
}

#[get("/image/<image_id>")]
pub fn image(image_id: String, conn: DbConn, auth: Authentication) -> Option<Template>
{
    use schema::horus_images::dsl::*;
    let image = horus_images.find(image_id).get_result::<HImage>(&*conn);

    if image.is_err() {
        return None;
    }

    let mut image = image.unwrap().with_displayable_date();

    if auth.get_userid() != image.owner {
        return None;
    }

    if image.title.is_none() {
        image.title = Some("Horus Image".to_string())
    }

    let path = if image.password == None {
        None
    } else {
        Some(::dbtools::s3::get_s3_presigned_url(image.filepath).unwrap())
    };

    let context = ManageImage {
        id: image.id,
        title: image.title.clone().unwrap(),
        page_title: image.title.clone().unwrap(),
        is_expiry: image.is_expiry,
        date_added: image.date_added,
        password: image.password,
        img_src: path,
        editable: true,
    };

    Some(Template::render("manage_image", &context))
}

#[get("/file/<file_id>")]
pub fn file(file_id: String, conn: DbConn, auth: Authentication) -> Option<Template>
{
    use schema::horus_files::dsl::*;
    let file = horus_files.find(file_id).get_result::<HFile>(&*conn);
    if file.is_err() {
        return None;
    }

    let file = file.unwrap();

    if auth.get_userid() != file.owner {
        return None;
    }

    let context = ManageFile {
        id: file.id,
        filename: file.filename.clone(),
        page_title: file.filename.clone(),
        is_expiry: file.is_expiry,
        date_added: format!("{}", file.date_added.format("%d %b %Y\nat %H:%M")),
        password: file.password.clone(),
        editable: false
    };

    Some(Template::render("manage_file", &context))
}

#[get("/paste/<paste_id>")]
pub fn paste(paste_id: String, conn: DbConn, auth: Authentication) -> Option<Template>
{
    use schema::horus_pastes::dsl::*;
    let paste = horus_pastes.find(paste_id).get_result::<HPaste>(&*conn);

    if paste.is_err() {
        return None;
    }
    let paste = paste.unwrap();

    if auth.get_userid() != paste.owner {
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
pub fn my_images_pageless() -> Redirect
{
    Redirect::to("images/0")
}

#[get("/videos")]
pub fn my_videos_pageless() -> Redirect
{
    Redirect::to("images/0")
}

#[get("/files")]
pub fn my_files_pageless() -> Redirect
{
    Redirect::to("images/0")
}

#[get("/pastes")]
pub fn my_pastes_pageless() -> Redirect
{
    Redirect::to("images/0")
}

#[get("/manage")]
pub fn base_redirect() -> Redirect
{
    Redirect::to("images/0")
}
