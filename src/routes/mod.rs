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
pub mod dist;

pub mod meta {
    use ::models::{LicenseKey, SessionToken};
    use ::DbConn;
    use rocket_contrib::Template;
    use rocket::response::{ status, Redirect, Failure, NamedFile };
    use std::path::Path;
    use std::collections::HashMap;

    const VERSION: &'static str = "2.4";
    const LATEST_PATH: &'static str = "live/binaries/";

    #[get("/version")]
    pub fn get_version(conn: DbConn) -> Result<String, status::Custom<String>> {
        use ::routes::dist;
        dist::get_version(Some("win64".to_string()), conn)
    }

    #[get("/changelogs")]
    pub fn changelogs() -> Option<Template> {
        // We don't actually need any context, we are just rendering from
        // a template for the telemetry data and the precompiled CSS, the data
        // itself is static in the template.
        let context: HashMap<String, String> = HashMap::new();
        Some(Template::render("changelog", &context))
    }

    #[get("/latest/<platform>", rank = 2)]
    pub fn get_latest_session(platform: String, conn: DbConn, session: SessionToken) -> Result<Redirect, Failure> {
        use ::routes::dist;
        
        dist::get_latest_sess(platform, conn, session)
    }

    #[get("/latest/<platform>", rank = 1)]
    pub fn get_latest(platform: String, conn: DbConn, apikey: LicenseKey) -> Result<Redirect, Failure> {
        use ::routes::dist;
        dist::get_latest(platform, conn, apikey)
    }
}
