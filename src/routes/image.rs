extern crate base64;

use std::path::Path;
use std::io::Read;

use chrono::{Local, NaiveDateTime};
#[allow(unused_imports)]
use diesel::{self, prelude::*};
use rocket::response::{status, NamedFile};
use rocket::data::Data;
use rocket::http::Status;
use rocket_contrib::json::Json;
use rocket_contrib::templates::Template;

use DbConn;
use dbtools;
use {contexts, conv};
use models::HImage;
use fields::{Authentication, PrivilegeLevel};
use forms::HImageChangesetForm;

#[get("/<image_id>")]
pub fn show(image_id: String, conn: DbConn) -> Option<Template>
{
    use schema::horus_images::dsl::*;

    let image = horus_images.find(&image_id).get_result::<HImage>(&*conn);

    if image.is_err() {
        return None;
    }
    let image = image.unwrap();
    let mut uri = String::from("https://s3.eu-central-1.amazonaws.com/horuscdn/live/images/");
    uri += &image.id.clone();
    uri += ".png";

    // Todo: Get this string programatically
    // And add width/height numbers, title, etc.
    let mut metatag = String::from("<meta property=\"og:image\" content=\"");
    metatag += &uri;
    metatag += "\" />";
    metatag += "<link rel=\"image_src\" href=\"";
    metatag += &uri;
    metatag += "\" />";

    let context = contexts::ShowImage {
        password: image.password.is_some(),
        item: image.with_displayable_date(),
        meta_tag: Some(metatag),
    };

    Some(Template::render("show_image", &context))
}

#[get("/full/<image_id>")]
pub fn full(image_id: String, conn: DbConn) -> Option<NamedFile>
{
    use schema::horus_images::dsl::*;
    let image = horus_images.find(image_id).get_result::<HImage>(&*conn);

    if image.is_err() {
        return None;
    }
    let image = image.unwrap();

    let image_path = Path::new(&image.filepath);
    NamedFile::open(image_path).ok()
}

#[get("/thumb/<image_id>")]
pub fn thumb(image_id: String, conn: DbConn) -> Option<NamedFile>
{
    full(image_id, conn)
}

/// `list` returns a paginated JSON array of HImage objects.
/// Pages start at index 0.
#[get("/<uid>/list/<page>")]
pub fn list(
    uid: i32,
    page: u32,
    auth: Authentication,
    conn: DbConn,
) -> Result<Json<Vec<HImage>>, status::Custom<()>>
{
    use schema::horus_images::dsl::*;

    if auth.get_userid() != uid && auth.get_privilege_level() == PrivilegeLevel::User {
        return Err(status::Custom(Status::Unauthorized, ()));
    }

    let images = horus_images
        .filter(owner.eq(uid))
        .order(date_added.desc())
        .limit(24)
        .offset((page * 24) as i64)
        .get_results::<HImage>(&*conn);

    if images.is_err() {
        return Err(status::Custom(Status::InternalServerError, ()));
    }

    Ok(Json(images.unwrap()))
}

#[delete("/<image_id>")]
pub fn delete(
    image_id: String,
    auth: Authentication,
    conn: DbConn,
) -> Result<status::Custom<()>, status::Custom<()>>
{
    use schema::horus_images::dsl::*;

    let image = horus_images.find(&image_id).get_result::<HImage>(&*conn);

    if image.is_err() {
        return Err(status::Custom(Status::NotFound, ()));
    }

    let image = image.unwrap();

    if auth.get_userid() != image.owner {
        return Err(status::Custom(Status::Unauthorized, ()));
    }

    delete_internal(image, conn)
}

fn delete_internal(image: HImage, conn: DbConn) -> Result<status::Custom<()>, status::Custom<()>>
{
    let s3result = dbtools::s3::delete_s3_object(&image.filepath);

    if s3result.is_err() {
        return Err(status::Custom(Status::ServiceUnavailable, ()));
    }

    let result = diesel::delete(&image).execute(&*conn);

    if result.is_err() {
        println!(
            "Database error while deleting image: {}",
            result.err().unwrap()
        );
        return Err(status::Custom(Status::InternalServerError, ()));
    }

    Ok(status::Custom(Status::Ok, ()))
}

fn new_img(
    img_data: Data,
    title: String,
    exp: Option<NaiveDateTime>,
    auth: Authentication,
    conn: DbConn,
) -> Result<status::Created<()>, status::Custom<()>>
{
    use schema::horus_images;
    let iid: String = dbtools::get_random_char_id(8);

    let pathstr = dbtools::get_path_image(&iid);

    let image = HImage::new(iid.clone(),
            Some(title),
            auth.get_userid(),
            pathstr.clone(),
            Local::now().naive_utc(),
            exp.is_some(),
            exp);

    // SAVE THE FILE THEN INSERT DB
    let img_data: Vec<u8> = img_data.open().bytes().map(|x| x.unwrap()).collect();

    // Removes the prefix
    let raw_img_data = base64::decode(&img_data[22..]);

    if raw_img_data.is_err() {
        println!("decode error");
        return Err(status::Custom(Status::BadRequest, ()));
    }

    let raw_img_data = raw_img_data.unwrap();

    let s3result = dbtools::s3::resource_to_s3(&pathstr, &raw_img_data);

    if s3result.is_err() {
        return Err(status::Custom(Status::ServiceUnavailable, ()));
    }

    let result = diesel::insert_into(horus_images::table)
        .values(&image)
        .get_result::<HImage>(&*conn);

    if result.is_err() {
        return Err(status::Custom(Status::InternalServerError, ()));
    }

    let result = result.unwrap();
    create_thumbnail_job(&image.id, &raw_img_data, image.owner);

    Ok(status::Created(
        String::from("/image/") + result.id.as_str(),
        None,
    ))
}

fn create_thumbnail_job(image_id: &str, image_data: &Vec<u8>, owner: i32)
{
    use models::{NewJob, JobPriority};
    // TODO

    let new_job = NewJob::new(owner,
                              "thumbnail:image".to_string(),
                              Some(image_data.to_vec()),
                              JobPriority::Normal);
    let queue_result = job_juggler::enqueue_job(new_job);

    if queue_result.is_err() {
        eprintln!("Could not enqueue thumbnail job: {}", queue_result.err().unwrap());
    }

    queue_result.unwrap();
}

#[post("/new", format = "image/png", data = "<img_data>")]
pub fn new(
    img_data: Data,
    auth: Authentication,
    conn: DbConn,
) -> Result<status::Created<()>, status::Custom<()>>
{
    new_img(img_data, String::from("Horus Image"), None, auth, conn)
}

/// <img_data> The base64 video data.
/// <expt> The expiration type 'minutes', 'hours', or 'days', optional.
/// <expd> The expiration duration, required if expt present.
#[post("/new/<expt>/<expd>", format = "image/png", data = "<img_data>")]
pub fn new_exp(
    img_data: Data,
    expt: Option<String>,
    expd: Option<usize>,
    auth: Authentication,
    conn: DbConn,
) -> Result<status::Created<()>, status::Custom<()>>
{
    if expt.is_some() && expd.is_some() {
        let exp = conv::get_dt_from_duration(expt.unwrap(), expd.unwrap() as isize);
        if exp.is_err() {
            return Err(status::Custom(Status::BadRequest, ()));
        }
        new_img(
            img_data,
            String::from("Horus Image"),
            Some(exp.unwrap()),
            auth,
            conn,
        )
    } else {
        new_img(img_data, String::from("Horus Image"), None, auth, conn)
    }
}

/// <img_data> The base64 video data.
/// <title> The title of the image, required.
/// <expt> The expiration type 'minutes', 'hours', or 'days', optional.
/// <expd> The expiration duration, required if expt present.
#[post("/new/<title>/<expt>/<expd>", format = "image/png", data = "<img_data>")]
pub fn new_titled(
    img_data: Data,
    title: String,
    expt: Option<String>,
    expd: Option<usize>,
    auth: Authentication,
    conn: DbConn,
) -> Result<status::Created<()>, status::Custom<()>>
{
    if expt.is_some() && expd.is_some() {
        let exp = conv::get_dt_from_duration(expt.unwrap(), expd.unwrap() as isize);
        if exp.is_err() {
            return Err(status::Custom(Status::BadRequest, ()));
        }
        new_img(img_data, title, Some(exp.unwrap()), auth, conn)
    } else {
        new_img(img_data, title, None, auth, conn)
    }
}

#[put("/<image_id>", format = "application/json", data = "<updated_values>")]
pub fn update(
    image_id: String,
    updated_values: Json<HImageChangesetForm>,
    auth: Authentication,
    conn: DbConn,
) -> Result<status::Accepted<()>, status::Custom<()>>
{
    use schema::horus_images::dsl::*;

    let img = horus_images
        .filter(id.eq(&image_id))
        .first::<HImage>(&*conn);

    if img.is_err() {
        return Err(status::Custom(Status::NotFound, ()));
    }
    let mut img = img.unwrap();

    if auth.get_userid() != img.owner {
        return Err(status::Custom(Status::Unauthorized, ()));
    }

    let img_update = updated_values.into_inner();
    let dt = conv::get_dt_from_duration(img_update.duration_type, img_update.duration_val);

    if !dt.is_err() {
        img.is_expiry = true;
        img.expiration_time = Some(dt.unwrap());
    }

    if let Some(_) = img_update.title {
        img.title = img_update.title;
    }

    let result = img.save_changes::<HImage>(&*conn);

    match result {
        Ok(_) => Ok(status::Accepted(None)),
        Err(_) => Err(status::Custom(Status::InternalServerError, ())),
    }
}

