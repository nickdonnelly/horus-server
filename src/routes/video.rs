extern crate diesel;
extern crate base64;
extern crate chrono;

use diesel::prelude::*;
use super::super::DbConn;
use super::super::dbtools;
use super::super::contexts;
use super::super::models::{LicenseKey, SessionToken, HVideo};
use super::super::forms::HVideoChangesetForm;
use rocket::response::{status, Failure, NamedFile};
use rocket::data::Data;
use rocket::http::Status;
use rocket_contrib::{Json, Template};
use self::chrono::Local;

use std::io::prelude::*;
use std::path::Path;

#[post("/new", format = "video/webm", data = "<vid_data>")]
pub fn new(
    vid_data: Data,
    apikey: LicenseKey,
    conn: DbConn)
    -> Result<status::Created<()>, Failure>
{
    use schema::horus_videos;
    let iid = dbtools::get_random_char_id(8);
    let pathstr = dbtools::get_path_video(&iid);

    let video = HVideo {
        id: iid.clone(),
        title: None,
        owner: apikey.get_owner(),
        filepath: pathstr.clone(),
        date_added: Local::now().naive_utc(),
        is_expiry: false,
        expiration_time: None,
    };

    let vid_data: Vec<u8> = vid_data.open()
        .bytes()
        .map(|x| x.unwrap())
        .collect();

    // 1 more character due too "webm" vs "png"
    let vid_data_decoded = base64::decode(&vid_data[23..]);

    if vid_data_decoded.is_err() {
        return Err(Failure(Status::BadRequest));
    }

    let vid_data_decoded = vid_data_decoded.unwrap();

    let s3result = dbtools::resource_to_s3(&pathstr, &vid_data_decoded);

    if s3result.is_err() {
        return Err(Failure(Status::ServiceUnavailable));
    }

    let result = diesel::insert(&video)
        .into(horus_videos::table)
        .get_result::<HVideo>(&*conn);

    if result.is_err() {
        return Err(Failure(Status::InternalServerError));
    }

    let result = result.unwrap();

    Ok(status::Created(String::from("/video/") + result.id.as_str(), None))
}

#[get("/<uid>/list/<page>")]
pub fn list(
    uid: i32,
    page: u32,
    apikey: LicenseKey,
    conn: DbConn)
    -> Result<Json<Vec<HVideo>>, Failure>
{
    use schema::horus_videos::dsl::*;

    if !apikey.belongs_to(uid) {
        println!("Unauthorized video list attempt by key: {}", apikey.key);
        return Err(Failure(Status::Unauthorized));
    }

    let videos = horus_videos
        .filter(owner.eq(uid))
        .order(date_added.desc())
        .limit(24)
        .offset((page * 24) as i64)
        .get_results::<HVideo>(&*conn);

    if videos.is_err() {
        println!(": {}", videos.err().unwrap());
        return Err(Failure(Status::InternalServerError));
    }

    Ok(Json(videos.unwrap()))
}

#[delete("/<vid_id>", rank = 2)]
pub fn delete_sessionless(
    vid_id: String,
    apikey: LicenseKey,
    conn: DbConn)
    -> Result<status::Custom<()>, Failure>
{
    use schema::horus_videos::dsl::*;

    let video = horus_videos
        .find(&vid_id)
        .get_result::<HVideo>(&*conn);

    if video.is_err() {
        return Err(Failure(Status::NotFound));
    }

    let video = video.unwrap();
    
    if !apikey.belongs_to(video.owner) {
        return Err(Failure(Status::Unauthorized));
    }

    delete_internal(video, conn)
}

#[delete("/<vid_id>")]
pub fn delete(
    vid_id: String,
    session: SessionToken,
    conn: DbConn)
    -> Result<status::Custom<()>, Failure>
{
    use schema::horus_videos::dsl::*;
    let video = horus_videos
        .find(&vid_id)
        .get_result::<HVideo>(&*conn);

    if video.is_err() {
        return Err(Failure(Status::NotFound));
    }

    let video = video.unwrap();
    
    if session.uid != video.owner {
        return Err(Failure(Status::Unauthorized));
    }

    delete_internal(video, conn)
}

fn delete_internal(
    video: HVideo,
    conn: DbConn)
    -> Result<status::Custom<()>, Failure>
{
    let s3result = dbtools::delete_s3_object(&video.filepath);

    if s3result.is_err() {
        return Err(Failure(Status::ServiceUnavailable));
    }

    let result = diesel::delete(&video).execute(&*conn);

    if result.is_err() {
        println!("Database error while deleting video: {}", result.err().unwrap());
        return Err(Failure(Status::InternalServerError));
    }

    Ok(status::Custom(Status::Ok, ()))
}

#[put("/<vid_id>", format = "application/json", data = "<updated_values>")]
pub fn update(
    vid_id: String,
    updated_values: Json<HVideoChangesetForm>,
    apikey: LicenseKey,
    conn: DbConn)
    -> Result<status::Accepted<()>, Failure>
{
    use schema::horus_videos::dsl::*;

    let vid = horus_videos.filter(id.eq(&vid_id))
        .first::<HVideo>(&*conn);
    if vid.is_err() {
        return Err(Failure(Status::NotFound));
    }
    
    let vid = vid.unwrap();

    if !apikey.belongs_to(vid.owner) {
        return Err(Failure(Status::Unauthorized));
    }

    let vid_update = updated_values.into_inner();
    let result = diesel::update(horus_videos.filter(id.eq(vid_id)))
        .set(&vid_update)
        .execute(&*conn);

    match result {
        Ok(_) => Ok(status::Accepted(None)),
        Err(_) => Err(Failure(Status::InternalServerError))
    }
}

#[get("/full/<vid_id>")]
pub fn full(
    vid_id: String,
    conn: DbConn)
    -> Option<NamedFile>
{
    use schema::horus_videos::dsl::*;
    let video = horus_videos.find(vid_id)
        .get_result::<HVideo>(&*conn);
    
    if video.is_err() {
        return None;
    }
    let video = video.unwrap();
    let video_path = Path::new(&video.filepath);
    NamedFile::open(video_path).ok()
}

#[get("/<vid_id>")]
pub fn show(
    vid_id: String,
    conn: DbConn)
    -> Option<Template>
{
    use schema::horus_videos::dsl::*;
    let video = horus_videos.find(&vid_id)
        .get_result::<HVideo>(&*conn);

    if video.is_err() {
        return None;
    }
    let video = video.unwrap();
    let mut metatag = String::from("<meta property=\"og:video\" content=\"https://s3.eu-central-1.amazonaws.com/horuscdn/live/videos/");
    metatag += &video.id.clone();
    metatag += ".webm\" />";
    let context = contexts::ShowVideo {
        item: video,
        meta_tag: Some(metatag),
    };

    Some(Template::render("show_video", &context))
}
