// These modules each contain a set of routes pertinent to their model name.
// CRUD models are present.

pub mod user;
pub mod jobs;
pub mod files;
pub mod manage;
pub mod image;
pub mod video;
pub mod paste;
pub mod key;

pub mod meta {
    use super::super::models::{LicenseKey, SessionToken};
    use rocket::response::NamedFile;
    use std::path::Path;

    const version: &'static str = "2.0.0";
    const latest_path: &'static str = "live/binaries/";
    
    #[get("/version")]
    pub fn get_version() -> String 
    {
        String::from(version)
    }

    #[get("/latest/<platform>")]
    pub fn get_latest_session(
        platform: String,
        _session: SessionToken)
        -> Option<NamedFile>
    {
        let pathstr = match platform.to_lowercase().as_str() {
            "linux" => String::from(latest_path) + "linux.zip",
            "win64" => String::from(latest_path) + "win64.zip",
            _ => String::new()
        };

        NamedFile::open(Path::new(&pathstr)).ok()
    }

    #[get("/latest/<platform>", rank = 2)]
    pub fn get_latest(
        platform: String,
        _apikey: LicenseKey)
        -> Option<NamedFile>
    {
        let pathstr = match platform.to_lowercase().as_str() {
            "linux" => String::from(latest_path) + "linux.zip",
            "win64" => String::from(latest_path) + "win64.zip",
            _ => String::new()
        };

        NamedFile::open(Path::new(&pathstr)).ok()
    }
}
