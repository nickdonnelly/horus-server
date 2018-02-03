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
    use rocket_contrib::Template;
    use rocket::response::NamedFile;
    use std::path::Path;
    use std::collections::HashMap;

    const VERSION: &'static str = "2.1.2";
    const LATEST_PATH: &'static str = "live/binaries/";
    
    #[get("/version")]
    pub fn get_version() -> String 
    {
        String::from(VERSION)
    }

    #[get("/changelogs")]
    pub fn changelogs() -> Option<Template>
    {
        // We don't actually need any context, we are just rendering from
        // a template for the telemetry data and the precompiled CSS, the data
        // itself is static in the template.
        let context: HashMap<String, String> = HashMap::new(); 
        Some(Template::render("changelog", &context))
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

    // This is for older versions that are pointing to the wrong
    // endpoint.
    #[get("/get_latestwin64")]
    pub fn get_latest_old_win(_apikey: LicenseKey)
        -> Option<NamedFile>
    {
        let pathstr = String::from(LATEST_PATH) + "win64.zip";
        NamedFile::open(Path::new(&pathstr)).ok()
    }

    #[get("/get_latestlinux")]
    pub fn get_latest_old_lin(_apikey: LicenseKey)
        -> Option<NamedFile>
    {
        let pathstr = String::from(LATEST_PATH) + "linux.zip";
        NamedFile::open(Path::new(&pathstr)).ok()
    }
}
