extern crate chrono;
extern crate base64;
extern crate diesel;

use self::chrono::NaiveDateTime;
use diesel::prelude::*;
use super::super::DbConn;
use super::super::dbtools;
use super::super::{contexts, conv};
use super::super::models::{LicenseKey,HImage,SessionToken};
use super::super::forms::HImageChangesetForm;
use rocket::response::{Failure, NamedFile, status};
use rocket::data::Data;
use rocket::http::Status;
use rocket_contrib::{Json, Template};
use self::chrono::Local;

use std::path::Path;
use std::io::Read;
use std::io::prelude::*;

#[get("/<image_id>")]
pub fn show(
    image_id: String,
    conn: DbConn)
    -> Option<Template>
{
    use schema::horus_images::dsl::*;

    let image = horus_images.find(&image_id)
        .get_result::<HImage>(&*conn);

    if image.is_err() {
        return None;
    }
    let image = image.unwrap();
    let mut metatag = String::from("<meta property=\"og:image\" content=\"https://s3.eu-central-1.amazonaws.com/horuscdn/live/images/");
    metatag += &image.id.clone();
    metatag += ".png\" />";

    let context = contexts::ShowImage {
        item: image,
        meta_tag: Some(metatag),
    };
    Some(Template::render("show_image", &context))
}

#[get("/full/<image_id>")]
pub fn full(
    image_id: String,
    conn: DbConn)
    -> Option<NamedFile>
{
    use schema::horus_images::dsl::*;
    let image = horus_images.find(image_id)
        .get_result::<HImage>(&*conn);

    if image.is_err() {
        return None;
    }
    let image = image.unwrap();

    let image_path = Path::new(&image.filepath);
    NamedFile::open(image_path).ok()
}

#[get("/thumb/<image_id>")]
pub fn thumb(
    image_id: String,
    conn: DbConn)
    -> Option<NamedFile>
{
    full(image_id, conn)
}

/// `list` returns a paginated JSON array of HImage objects.
/// Pages start at index 0.
#[get("/<uid>/list/<page>")]
pub fn list(
    uid: i32,
    page: u32,
    apikey: LicenseKey,
    conn: DbConn)
    -> Result<Json<Vec<HImage>>, Failure>
{
    use schema::horus_images::dsl::*;

    if !apikey.belongs_to(uid) {
        return Err(Failure(Status::Unauthorized));
    }

    let images = horus_images
        .filter(owner.eq(uid))
        .order(date_added.desc())
        .limit(24)
        .offset((page * 24) as i64)
        .get_results::<HImage>(&*conn);

    if images.is_err() {
        return Err(Failure(Status::InternalServerError));
    }

    Ok(Json(images.unwrap()))
}

#[delete("/<image_id>")]
pub fn delete(
    image_id: String,
    session: SessionToken,
    conn: DbConn)
    -> Result<status::Custom<()>, Failure>
{
    use schema::horus_images::dsl::*;

    let image = horus_images
        .find(&image_id)
        .get_result::<HImage>(&*conn);

    if image.is_err() {
        return Err(Failure(Status::NotFound));
    }

    let image = image.unwrap();
    
    if session.uid != image.owner {
        return Err(Failure(Status::Unauthorized));
    }

    delete_internal(image, conn)
}

#[delete("/<image_id>", rank=2)]
pub fn delete_sessionless(
    image_id: String,
    apikey: LicenseKey,
    conn: DbConn)
    -> Result<status::Custom<()>, Failure>
{
    use schema::horus_images::dsl::*;

    let image = horus_images
        .find(&image_id)
        .get_result::<HImage>(&*conn);

    if image.is_err() {
        return Err(Failure(Status::NotFound));
    }

    let image = image.unwrap();
    
    if !apikey.belongs_to(image.owner) {
        return Err(Failure(Status::Unauthorized));
    }


    delete_internal(image, conn)
}

fn delete_internal(
    image: HImage,
    conn: DbConn) 
    -> Result<status::Custom<()>, Failure>
{
    let s3result = dbtools::delete_s3_object(&image.filepath);

    if s3result.is_err() {
        return Err(Failure(Status::ServiceUnavailable));
    }

    let result = diesel::delete(&image).execute(&*conn);

    if result.is_err() {
        println!("Database error while deleting image: {}", result.err().unwrap());
        return Err(Failure(Status::InternalServerError));
    }

    Ok(status::Custom(Status::Ok, ()))
}


fn new_img(
    img_data: Data,
    title: String,
    exp: Option<NaiveDateTime>,
    apikey: LicenseKey,
    conn: DbConn)
    -> Result<status::Created<()>, Failure>
{
    use schema::horus_images;
    let iid: String = dbtools::get_random_char_id(8);

    let pathstr = dbtools::get_path_image(&iid);

    let image = HImage {
        id: iid.clone(),
        title: Some(title),
        owner: apikey.get_owner(),
        filepath: pathstr.clone(),
        date_added: Local::now().naive_utc(),
        is_expiry: exp.is_some(),
        expiration_time: exp,
    };
    // SAVE THE FILE THEN INSERT DB
    let img_data: Vec<u8> = img_data.open()
        .bytes()
        .map(|x| x.unwrap())
        .collect();

    // Removes the prefix
    let raw_img_data = base64::decode(&img_data[22..]);

    if raw_img_data.is_err() {
        println!("decode error");
        return Err(Failure(Status::BadRequest));
    }

    let raw_img_data = raw_img_data.unwrap();

    let s3result = dbtools::resource_to_s3(&pathstr, &raw_img_data);

    if s3result.is_err() {
        return Err(Failure(Status::ServiceUnavailable));
    }

    let result = diesel::insert_into(horus_images::table)
        .values(&image)
        .get_result::<HImage>(&*conn);
    
    if result.is_err() {
        return Err(Failure(Status::InternalServerError));
    }
    
    let result = result.unwrap();

    Ok(status::Created(String::from("/image/") + result.id.as_str(), None))
}

#[post("/new", format="image/png", data = "<img_data>")]
pub fn new(
    img_data: Data,
    apikey: LicenseKey,
    conn: DbConn)
    -> Result<status::Created<()>, Failure>
{
    new_img(img_data, String::from("Horus Image"), None, apikey, conn)
}

/// <img_data> The base64 video data.
/// <expt> The expiration type 'minutes', 'hours', or 'days', optional.
/// <expd> The expiration duration, required if expt present.
#[post("/new/<expt>/<expd>", format="image/png", data = "<img_data>")]
pub fn new_exp(
    img_data: Data,
    expt: Option<String>,
    expd: Option<usize>,
    apikey: LicenseKey,
    conn: DbConn)
    -> Result<status::Created<()>, Failure>
{
    if expt.is_some() && expd.is_some() {
        let exp = conv::get_dt_from_duration(expt.unwrap(), expd.unwrap() as isize);
        if exp.is_err() {
            return Err(Failure(Status::BadRequest));
        }
        new_img(img_data, String::from("Horus Image"), Some(exp.unwrap()), apikey, conn)
    } else {
        new_img(img_data, String::from("Horus Image"), None, apikey, conn)
    }
}

/// <img_data> The base64 video data.
/// <title> The title of the image, required.
/// <expt> The expiration type 'minutes', 'hours', or 'days', optional.
/// <expd> The expiration duration, required if expt present.
#[post("/new/<title>/<expt>/<expd>", format="image/png", data="<img_data>")]
pub fn new_titled(
    img_data: Data,
    title: String,
    expt: Option<String>,
    expd: Option<usize>,
    apikey: LicenseKey,
    conn: DbConn)
    -> Result<status::Created<()>, Failure>
{
    if expt.is_some() && expd.is_some() {
        let exp = conv::get_dt_from_duration(expt.unwrap(), expd.unwrap() as isize);
        if exp.is_err() {
            return Err(Failure(Status::BadRequest));
        }
        new_img(img_data, title, Some(exp.unwrap()), apikey, conn)
    } else {
        new_img(img_data, title, None, apikey, conn)
    }
}

#[put("/<image_id>", format = "application/json", data = "<updated_values>")]
pub fn update(
    image_id: String,
    updated_values: Json<HImageChangesetForm>,
    apikey: LicenseKey,
    conn: DbConn)
    -> Result<status::Accepted<()>, Failure>
{
    use schema::horus_images::dsl::*;

    let img = horus_images.filter(id.eq(&image_id))
        .first::<HImage>(&*conn);

    if img.is_err() {
        return Err(Failure(Status::NotFound));
    }
    let mut img = img.unwrap();

    if !apikey.belongs_to(img.owner) {
        return Err(Failure(Status::Unauthorized));
    }

    let img_update = updated_values.into_inner();
    let dt = conv::get_dt_from_duration(img_update.duration_type, img_update.duration_val);
    
    if !dt.is_err() {
        img.is_expiry = true;
        img.expiration_time = Some(dt.unwrap());
    }

    let result = img.save_changes::<HImage>(&*conn);

    match result {
        Ok(_) => Ok(status::Accepted(None)),
        Err(_) => Err(Failure(Status::InternalServerError)),
    }
}

