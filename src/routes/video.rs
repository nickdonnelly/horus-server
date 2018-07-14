extern crate base64;

use std::io::prelude::*;
use std::path::Path;

use chrono::{Local, NaiveDateTime};
use diesel::{self, prelude::*};
use rocket::response::{status, Failure, NamedFile};
use rocket::data::Data;
use rocket::http::Status;
use rocket_contrib::{Json, Template};

use DbConn;
use dbtools;
use {contexts, conv};
use models::HVideo;
use forms::HVideoChangesetForm;
use fields::Authentication;

fn new_vid(
    vid_data: Data,
    title: String,
    exp: Option<NaiveDateTime>,
    auth: Authentication,
    conn: DbConn,
) -> Result<status::Created<()>, Failure>
{
    use schema::horus_videos;
    let iid = dbtools::get_random_char_id(8);
    let pathstr = dbtools::get_path_video(&iid);

    let video = HVideo {
        id: iid.clone(),
        title: Some(title),
        owner: auth.get_userid(),
        filepath: pathstr.clone(),
        date_added: Local::now().naive_utc(),
        is_expiry: exp.is_some(),
        expiration_time: exp,
        password: None
    };

    let vid_data: Vec<u8> = vid_data.open().bytes().map(|x| x.unwrap()).collect();

    if vid_data.len() < 23 {
        return Err(Failure(Status::BadRequest));
    }

    // 1 more character due too "webm" vs "png"
    let vid_data_decoded = base64::decode(&vid_data[23..]);

    if vid_data_decoded.is_err() {
        eprintln!(
            "Couldn't decode webm data: {}",
            vid_data_decoded.err().unwrap()
        );
        return Err(Failure(Status::BadRequest));
    }

    let vid_data_decoded = vid_data_decoded.unwrap();

    let s3result = dbtools::s3::resource_to_s3(&pathstr, &vid_data_decoded);

    if s3result.is_err() {
        return Err(Failure(Status::ServiceUnavailable));
    }

    let result = diesel::insert_into(horus_videos::table)
        .values(&video)
        .get_result::<HVideo>(&*conn);

    if result.is_err() {
        return Err(Failure(Status::InternalServerError));
    }

    let result = result.unwrap();

    Ok(status::Created(
        String::from("/video/") + result.id.as_str(),
        None,
    ))
}

#[post("/new", format = "video/webm", data = "<vid_data>")]
pub fn new(
    vid_data: Data,
    auth: Authentication,
    conn: DbConn,
) -> Result<status::Created<()>, Failure>
{
    new_vid(vid_data, String::from("Horus Video"), None, auth, conn)
}

/// <vid_data> The base64 video data.
/// <expt> The expiration type 'minutes', 'hours', or 'days', optional.
/// <expd> The expiration duration, required if expt present.
#[post("/new/<expt>/<expd>", format = "video/webm", data = "<vid_data>")]
pub fn new_exp(
    vid_data: Data,
    expt: Option<String>,
    expd: Option<usize>,
    auth: Authentication,
    conn: DbConn,
) -> Result<status::Created<()>, Failure>
{
    if expt.is_some() && expd.is_some() {
        let exp = conv::get_dt_from_duration(expt.unwrap(), expd.unwrap() as isize);
        if exp.is_err() {
            return Err(Failure(Status::BadRequest));
        }
        new_vid(
            vid_data,
            String::from("Horus Video"),
            Some(exp.unwrap()),
            auth,
            conn,
        )
    } else {
        new_vid(vid_data, String::from("Horus Video"), None, auth, conn)
    }
}

/// <vid_data> The base64 video data.
/// <title> The title of the image, required.
/// <expt> The expiration type 'minutes', 'hours', or 'days', optional.
/// <expd> The expiration duration, required if expt present.
// TODO: change /expt/expd to ?<exp> of type Expiration which we can use #[derive(FromForm)] on.
// Do this for all of the resources.
#[post("/new/<title>/<expt>/<expd>", format = "video/webm", data = "<vid_data>")]
pub fn new_titled(
    vid_data: Data,
    title: String,
    expt: Option<String>,
    expd: Option<usize>,
    auth: Authentication,
    conn: DbConn,
) -> Result<status::Created<()>, Failure>
{
    if expt.is_some() && expd.is_some() {
        let exp = conv::get_dt_from_duration(expt.unwrap(), expd.unwrap() as isize);
        if exp.is_err() {
            return Err(Failure(Status::BadRequest));
        }
        new_vid(vid_data, title, Some(exp.unwrap()), auth, conn)
    } else {
        new_vid(vid_data, title, None, auth, conn)
    }
}

#[get("/<uid>/list/<page>")]
pub fn list(
    uid: i32,
    page: u32,
    auth: Authentication,
    conn: DbConn,
) -> Result<Json<Vec<HVideo>>, Failure>
{
    use schema::horus_videos::dsl::*;

    if auth.get_userid() != uid {
        println!(
            "Unauthorized video list attempt by auth with userid: {}",
            auth.get_userid()
        );
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

#[delete("/<vid_id>")]
pub fn delete(
    vid_id: String,
    auth: Authentication,
    conn: DbConn,
) -> Result<status::Custom<()>, Failure>
{
    use schema::horus_videos::dsl::*;
    let video = horus_videos.find(&vid_id).get_result::<HVideo>(&*conn);

    if video.is_err() {
        return Err(Failure(Status::NotFound));
    }

    let video = video.unwrap();

    if auth.get_userid() != video.owner {
        return Err(Failure(Status::Unauthorized));
    }

    delete_internal(video, conn)
}

fn delete_internal(video: HVideo, conn: DbConn) -> Result<status::Custom<()>, Failure>
{
    let s3result = dbtools::s3::delete_s3_object(&video.filepath);

    if s3result.is_err() {
        return Err(Failure(Status::ServiceUnavailable));
    }

    let result = diesel::delete(&video).execute(&*conn);

    if result.is_err() {
        println!(
            "Database error while deleting video: {}",
            result.err().unwrap()
        );
        return Err(Failure(Status::InternalServerError));
    }

    Ok(status::Custom(Status::Ok, ()))
}

#[put("/<vid_id>", format = "application/json", data = "<updated_values>")]
pub fn update(
    vid_id: String,
    updated_values: Json<HVideoChangesetForm>,
    auth: Authentication,
    conn: DbConn,
) -> Result<status::Accepted<()>, Failure>
{
    use schema::horus_videos::dsl::*;

    let vid = horus_videos.filter(id.eq(&vid_id)).first::<HVideo>(&*conn);

    if vid.is_err() {
        return Err(Failure(Status::NotFound));
    }

    let mut vid = vid.unwrap();

    if auth.get_userid() != vid.owner {
        return Err(Failure(Status::Unauthorized));
    }

    let vid_update = updated_values.into_inner();
    let dt = conv::get_dt_from_duration(vid_update.duration_type, vid_update.duration_val);

    if !dt.is_err() {
        vid.is_expiry = true;
        vid.expiration_time = Some(dt.unwrap());
    }

    if let Some(t) = vid_update.title {
        vid.title = Some(t);
    }

    let result = vid.save_changes::<HVideo>(&*conn);
    match result {
        Ok(_) => Ok(status::Accepted(None)),
        Err(_) => Err(Failure(Status::InternalServerError)),
    }
}

#[get("/full/<vid_id>")]
pub fn full(vid_id: String, conn: DbConn) -> Option<NamedFile>
{
    use schema::horus_videos::dsl::*;
    let video = horus_videos.find(vid_id).get_result::<HVideo>(&*conn);

    if video.is_err() {
        return None;
    }
    let video = video.unwrap();
    let video_path = Path::new(&video.filepath);
    NamedFile::open(video_path).ok()
}

#[get("/<vid_id>")]
pub fn show(vid_id: String, conn: DbConn) -> Option<Template>
{
    use schema::horus_videos::dsl::*;
    let video = horus_videos.find(&vid_id).get_result::<HVideo>(&*conn);

    if video.is_err() {
        return None;
    }
    let video = video.unwrap();
    let mut metatag = String::from("<meta property=\"og:video\" content=\"https://s3.eu-central-1.amazonaws.com/horuscdn/live/videos/");
    metatag += &video.id.clone();
    metatag += ".webm\" />";
    let context = contexts::ShowVideo {
        password: video.password.is_some(),
        item: video,
        meta_tag: Some(metatag),
    };

    Some(Template::render("show_video", &context))
}
