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
pub mod http_errors;
pub mod password;

pub mod meta
{
    use std::collections::HashMap;

    use rocket_contrib::Template;

    #[get("/changelogs")]
    pub fn changelogs() -> Option<Template>
    {
        // We don't actually need any context, we are just rendering from
        // a template for the telemetry data and the precompiled CSS, the data
        // itself is static in the template.
        let context: HashMap<String, String> = HashMap::new();
        Some(Template::render("changelog", &context))
    }
}
