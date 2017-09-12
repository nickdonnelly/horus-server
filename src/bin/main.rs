#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate horus_server;
extern crate rocket;
extern crate rocket_contrib;

use horus_server::*;
use rocket_contrib::Template;

fn main() {
    check_dirs();
    rocket::ignite()
        .attach(Template::fairing())
        .mount("/user", routes![self::routes::user::show])
        .mount("/user", routes![self::routes::user::update])
        .mount("/license", routes![self::routes::key::validity_check])
        .mount("/key", routes![self::routes::key::issue])
        .mount("/paste", routes![self::routes::paste::new])
        .mount("/paste", routes![self::routes::paste::show])
        .mount("/paste", routes![self::routes::paste::delete])
        .mount("/image", routes![self::routes::image::new])
        .manage(self::dbtools::init_pool())
        .launch();
}

fn check_dirs() {
    use std::fs;
    use std::path::Path;

    let mut directories: Vec<String> = Vec::new();
    directories.push("live/images/".to_string());
    directories.push("live/images/thumbnails".to_string());
    directories.push("live/videos/".to_string());
    directories.push("live/videos/thumbnails".to_string());
    directories.push("live/files/".to_string());
    
    for dirstr in directories {
        let path: &Path = Path::new(&dirstr);
        if !path.exists() {
            fs::create_dir_all(path).unwrap();
        }
    }
}
