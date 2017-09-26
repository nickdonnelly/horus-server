extern crate chrono;
extern crate diesel;

use diesel::prelude::*;
use rocket::request::Request;
use rocket::response::{status, NamedFile, Failure, Response, Responder};
use rocket::http::{ContentType, Status};
use rocket::data::Data;
use rocket_contrib::{Json, Template};
use super::super::models::{HFile, LicenseKey, SessionToken};
use super::super::DbConn;
use super::super::dbtools;
use super::super::fields::FileName;
use self::chrono::Local;

use std::path::{Path, PathBuf};
use std::io::Read;
use std::io::prelude::*;

pub struct DownloadableFile
{
    pub afile: NamedFile, 
    pub name: FileName
}

#[get("/<file_id>")]
pub fn get(
    file_id: String,
    conn: DbConn)
    -> Option<Template>
{
    use schema::horus_files::dsl::*;
    
    let hfile = horus_files.find(&file_id)
        .get_result::<HFile>(&*conn);

    if hfile.is_err() {
        return None;
    }
    let mut hfile = hfile.unwrap();
    hfile.download_counter = Some(hfile.download_counter.unwrap() + 1);
    hfile.save_changes::<HFile>(&*conn); // ignore the warning coming from this.
                                         // Success is not critical

    //let fname = FileName(hfile.filename);
    
    Some(Template::render("show_file", &hfile))
}

#[get("/<uid>/list/<page>")]
pub fn list(
    uid: i32,
    page: u32,
    session: SessionToken,
    conn: DbConn)
    -> Result<Json<Vec<HFile>>, Failure>
{
    use schema::horus_files::dsl::*;

    if session.uid != uid {
        return Err(Failure(Status::Unauthorized));
    }

    let files = horus_files
        .filter(owner.eq(uid))
        .order(date_added.desc())
        .limit(48)
        .offset((page * 48) as i64)
        .get_results::<HFile>(&*conn);

    if files.is_err() {
        println!("File selection failed with error: {}", files.err().unwrap());
        return Err(Failure(Status::InternalServerError));
    }

    Ok(Json(files.unwrap()))
}

#[post("/new", format="application/octet-stream", data = "<file_data>")]
pub fn new(
    file_data: Data,
    file_name: FileName,
    apikey: LicenseKey,
    conn: DbConn)
    -> Result<status::Created<()>, Failure>
{
    use schema::horus_files;
    let fid: String = dbtools::get_random_char_id(8);
    let pathstr = dbtools::get_path_file(&fid);

    let hfile = HFile {
        id: fid.clone(),
        owner: apikey.get_owner(),
        filename: file_name.0,
        filepath: pathstr.clone(),
        date_added: Local::now().naive_utc(),
        is_expiry: false,
        expiration_time: None,
        download_counter: None, // defaults 0
    };

    let file_data: Vec<u8> = file_data.open()   
        .bytes()
        .map(|x| x.unwrap())
        .collect();
    // No need to decode as we are getting raw bytes through an octet-stream, no base64

    let s3result = dbtools::resource_to_s3(&pathstr, &file_data);

    if s3result.is_err() {
        return Err(Failure(Status::ServiceUnavailable));
    }

    let result = diesel::insert(&hfile)
        .into(horus_files::table)
        .get_result::<HFile>(&*conn);

    if result.is_err() {
        return Err(Failure(Status::BadRequest));
    }
    let result = result.unwrap();
    Ok(status::Created(String::from("/file/") + result.id.as_str(), None))
}

#[delete("/<file_id>")]
pub fn delete(
    file_id: String,
    session: SessionToken,
    conn: DbConn)
    -> Result<status::Custom<()>, Failure>
{
    use schema::horus_files::dsl::*;
    let hfile = horus_files
        .find(&file_id)
        .get_result::<HFile>(&*conn);
    if hfile.is_err() {
        return Err(Failure(Status::NotFound));
    }
    let hfile = hfile.unwrap();

    if session.uid != hfile.owner {
        return Err(Failure(Status::Unauthorized));
    }

    delete_internal(hfile, conn)
}

#[delete("/<file_id>", rank = 2)]
pub fn delete_sessionless(
    file_id: String,
    apikey: LicenseKey,
    conn: DbConn)
    -> Result<status::Custom<()>, Failure>
{
    use schema::horus_files::dsl::*;
    let hfile = horus_files
        .find(&file_id)
        .get_result::<HFile>(&*conn);
    if hfile.is_err() {
        return Err(Failure(Status::NotFound));
    }
    let hfile = hfile.unwrap();

    if !apikey.belongs_to(hfile.owner) {
        return Err(Failure(Status::Unauthorized));
    }


    delete_internal(hfile, conn)
}

fn delete_internal(
    hfile: HFile,
    conn: DbConn)
    -> Result<status::Custom<()>, Failure>
{
    let result = diesel::delete(&hfile).execute(&*conn);
    if result.is_err() {
        println!("Database error while deleting image: {}", result.err().unwrap());
        return Err(Failure(Status::InternalServerError));
    }

    Ok(status::Custom(Status::Ok, ()))
}

impl Responder<'static> for DownloadableFile {
    fn respond_to(self, _: &Request) -> Result<Response<'static>, Status> {
        let mut response = Response::new();
        if let Some(ext) = self.afile.path().extension() {
            let ext_string = ext.to_string_lossy().to_lowercase();
            if let Some(content_type) = ContentType::from_extension(&ext_string) {
                response.set_header(content_type);
            }
        }

        let mut cdstr = String::from("attachment; filename=\"");
        cdstr += self.name.0.as_str();
        cdstr += "\"";
        response.set_raw_header("content-disposition", cdstr);
        response.set_streamed_body(self.afile.take_file());
        Ok(response)
    }
}

// This is not to do with files as user-uploaded bits. This
// just serves the static assets for the manage page.
#[get("/<file..>")]
fn static_asset(file: PathBuf) -> Option<NamedFile> 
{
    NamedFile::open(Path::new("static/").join(file)).ok()
}
