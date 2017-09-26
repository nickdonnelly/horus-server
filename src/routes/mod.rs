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

    const VERSION: &'static str = "2.0.0";
    const LATEST_PATH: &'static str = "live/binaries/";
    
    #[get("/version")]
    pub fn get_version() -> String 
    {
        String::from(VERSION)
    }

    #[get("/changelogs")]
    pub fn changelogs() -> Option<Template>
    {
        // TODO
        None
    }

    #[get("/latest/<platform>")]
    pub fn get_latest_session(
        platform: String,
        _session: SessionToken)
        -> Option<NamedFile>
    {
        let pathstr = match platform.to_lowercase().as_str() {
            "linux" => String::from(LATEST_PATH) + "linux.zip",
            "win64" => String::from(LATEST_PATH) + "win64.zip",
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
            "linux" => String::from(LATEST_PATH) + "linux.zip",
            "win64" => String::from(LATEST_PATH) + "win64.zip",
            _ => String::new()
        };

        NamedFile::open(Path::new(&pathstr)).ok()
    }
}
