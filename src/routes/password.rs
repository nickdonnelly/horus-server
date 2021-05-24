use std::boxed::Box;

use diesel::prelude::*;
use diesel::pg::PgConnection;
use rocket::request::{self, FromRequest, Request};
use rocket::response::{ Responder, status };
use rocket::http::Status;
use rocket::Outcome;

use dbtools::s3;
use fields::Authentication;
use models::traits::passwordable::Passwordable;
use DbConn;

/// The list of types that can be passworded derived from a request header
pub enum PasswordableResource {
    Image,
    Video,
    File 
}

/// Returns a signed S3 link to the resource if the password is correct.
#[post("/<res_id>", format="text/plain", data="<submitted_password>")]
pub fn check(
    res_type: PasswordableResource,
    res_id: String,
    submitted_password: String,
    conn: DbConn) 
    -> Result<status::Custom<String>, status::Custom<()>>
{
    let resource = get_passwordable_resource_by_id(res_id, res_type, &*conn);

    let resource = match resource {
        Some(r) => r,
        _       => return Err(status::Custom(Status::NotFound, ()))
    };

    if resource.check_password(submitted_password, &*conn) {
        let signed_location = s3::get_s3_presigned_url(resource.get_s3_location());

        match signed_location {
            Ok(link) => Ok(status::Custom(Status::Ok, link)),
            Err(_) => Err(status::Custom(Status::InternalServerError, ()))
        }
    } else {
        Err(status::Custom(Status::Unauthorized, ()))
    }
}

#[put("/<res_id>", format="text/plain", data="<submitted_password>")]
pub fn set(
    res_type: PasswordableResource,
    res_id: String,
    submitted_password: String,
    auth: Authentication,
    conn: DbConn)
    -> Result<status::Accepted<()>, status::Custom<()>>
{
    let resource = get_passwordable_resource_by_id(res_id.clone(), res_type, &*conn);
    let mut resource = match resource {
        Some(r) => r,
        None => return Err(status::Custom(Status::NotFound, ()))
    };

    if resource.owner() != auth.get_userid() {
        return Err(status::Custom(Status::Unauthorized, ()));
    }

    let submitted_password: Option<String> = match submitted_password.as_str() {
        "" => None,
        other => Some(other.to_string())
    };

    let set_result = resource.set_password(submitted_password.clone(), &*conn);

    if set_result.is_some() {
        eprintln!("Error changing password for {}: {}", res_id, set_result.unwrap());
        return Err(status::Custom(Status::InternalServerError, ()));
    }

    let s3_result = if submitted_password == None {
        s3::publicize_s3_resource(&resource.get_s3_location())
    } else {
        s3::privatize_s3_resource(&resource.get_s3_location())
    };


    match s3_result {
        Ok(()) => Ok(status::Accepted(None)),
        Err(_) => Err(status::Custom(Status::InternalServerError, ()))
    }
}

fn get_passwordable_resource_by_id(
    res_id: String,
    res_type: PasswordableResource,
    conn: &PgConnection)
    -> Option<Box<dyn Passwordable>>
{
    match res_type {
        PasswordableResource::Image => {
            use ::schema::horus_images::dsl::*;
            use ::models::HImage;
            let img = horus_images.find(res_id).get_result::<HImage>(conn);
            if img.is_err() { return None }
            Some(Box::new(img.unwrap()))
        },
        PasswordableResource::Video => {
            use ::schema::horus_videos::dsl::*;
            use ::models::HVideo;
            let vid = horus_videos.find(res_id).get_result::<HVideo>(conn);
            if vid.is_err() { return None }
            Some(Box::new(vid.unwrap()))
        },
        PasswordableResource::File => {
            use ::schema::horus_files::dsl::*;
            use ::models::HFile;
            let file = horus_files.find(res_id).get_result::<HFile>(conn);
            if file.is_err() { return None }
            Some(Box::new(file.unwrap()))
        }
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for PasswordableResource {
    type Error = ();

    fn from_request(request: &'a Request<'r>) 
        -> request::Outcome<PasswordableResource, Self::Error>
    {
        let header = request.headers().get_one("horus-resource-type");

        if header.is_none() {
            return Outcome::Failure((Status::BadRequest, ()));
        }

        let header = header.unwrap();

        match header.to_lowercase().trim() {
            "image" => Outcome::Success(PasswordableResource::Image),
            "video" =>  Outcome::Success(PasswordableResource::Video),
            "file" =>  Outcome::Success(PasswordableResource::File),
            _ => Outcome::Failure((Status::BadRequest, ()))
        }
    }

}
