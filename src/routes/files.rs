use std::path::{Path, PathBuf};
use std::io::Read;

use diesel;
use chrono::{Local, NaiveDateTime};
#[allow(unused_imports)]
use diesel::prelude::*;
use rocket::request::Request;
use rocket::response::{status, Failure, NamedFile, Responder, Response};
use rocket::http::{ContentType, Status};
use rocket::data::Data;
use rocket_contrib::{Json, Template};

use models::HFile;
use fields::{Authentication, PrivilegeLevel};
use DbConn;
use {conv, dbtools};
use fields::FileName;

pub struct DownloadableFile
{
    pub afile: NamedFile,
    pub name: FileName,
}

#[get("/<file_id>")]
pub fn get(file_id: String, conn: DbConn) -> Option<Template>
{
    use schema::horus_files::dsl::*;

    let hfile = horus_files.find(&file_id).get_result::<HFile>(&*conn);

    if hfile.is_err() {
        return None;
    }
    let mut hfile = hfile.unwrap();
    // TODO, make this an async javascript change, visiting the page doesn't imply they click dl.
    hfile.download_counter = Some(hfile.download_counter.unwrap() + 1);
    hfile.save_changes::<HFile>(&*conn).unwrap();

    Some(Template::render("show_file", &hfile))
}

#[get("/<uid>/list/<page>")]
pub fn list(
    uid: i32,
    page: u32,
    auth: Authentication,
    conn: DbConn,
) -> Result<Json<Vec<HFile>>, Failure>
{
    use schema::horus_files::dsl::*;

    if auth.get_userid() != uid && auth.get_privilege_level() == PrivilegeLevel::User {
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

#[post("/new", format = "application/octet-stream", data = "<file_data>")]
pub fn new(
    file_data: Data,
    file_name: FileName,
    auth: Authentication,
    conn: DbConn,
) -> Result<status::Created<()>, Failure>
{
    new_file(file_data, file_name, None, auth, conn)
}

#[post("/new/<expt>/<expd>", format = "application/octet-stream", data = "<file_data>")]
pub fn new_exp(
    file_data: Data,
    file_name: FileName,
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
        new_file(file_data, file_name, Some(exp.unwrap()), auth, conn)
    } else {
        new_file(file_data, file_name, None, auth, conn)
    }
}
pub fn new_file(
    file_data: Data,
    file_name: FileName,
    expire_time: Option<NaiveDateTime>,
    auth: Authentication,
    conn: DbConn,
) -> Result<status::Created<()>, Failure>
{
    use schema::horus_files;
    let fid: String = dbtools::get_random_char_id(8);
    let pathstr = dbtools::get_path_file(&fid);

    let hfile = HFile {
        id: fid.clone(),
        owner: auth.get_userid(),
        filename: file_name.0,
        filepath: pathstr.clone(),
        date_added: Local::now().naive_utc(),
        is_expiry: expire_time.is_some(),
        expiration_time: expire_time,
        download_counter: None, // defaults 0
    };

    let file_data: Vec<u8> = file_data.open().bytes().map(|x| x.unwrap()).collect();

    // No need to decode as we are getting raw bytes through an octet-stream, no base64
    let s3result = dbtools::s3::resource_to_s3_named(&hfile.filename, &pathstr, &file_data);

    if s3result.is_err() {
        return Err(Failure(Status::ServiceUnavailable));
    }

    let result = diesel::insert_into(horus_files::table)
        .values(&hfile)
        .get_result::<HFile>(&*conn);

    if result.is_err() {
        return Err(Failure(Status::BadRequest));
    }
    let result = result.unwrap();
    Ok(status::Created(
        String::from("/file/") + result.id.as_str(),
        None,
    ))
}

#[delete("/<file_id>")]
pub fn delete(
    file_id: String,
    auth: Authentication,
    conn: DbConn,
) -> Result<status::Custom<()>, Failure>
{
    use schema::horus_files::dsl::*;
    let hfile = horus_files.find(&file_id).get_result::<HFile>(&*conn);
    if hfile.is_err() {
        return Err(Failure(Status::NotFound));
    }
    let hfile = hfile.unwrap();

    if auth.get_userid() != hfile.owner {
        return Err(Failure(Status::Unauthorized));
    }

    delete_internal(hfile, conn)
}

fn delete_internal(hfile: HFile, conn: DbConn) -> Result<status::Custom<()>, Failure>
{
    let s3result = dbtools::s3::delete_s3_object(&hfile.filepath);

    if s3result.is_err() {
        return Err(Failure(Status::ServiceUnavailable));
    }

    let result = diesel::delete(&hfile).execute(&*conn);
    if result.is_err() {
        println!(
            "Database error while deleting image: {}",
            result.err().unwrap()
        );
        return Err(Failure(Status::InternalServerError));
    }

    Ok(status::Custom(Status::Ok, ()))
}

impl Responder<'static> for DownloadableFile
{
    fn respond_to(self, _: &Request) -> Result<Response<'static>, Status>
    {
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
