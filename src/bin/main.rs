#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate horus_server;
extern crate rocket;
extern crate rocket_contrib;

use horus_server::*;
use rocket_contrib::Template;
use rocket::response::NamedFile;
use std::path::Path;

fn main() {
    use self::routes::*;
    check_dirs();
    rocket::ignite()
        .attach(Template::fairing())
        .mount("/user", routes![user::show, user::update])
        .mount("/key", routes![key::validity_check, key::issue])
        .mount("/paste", routes![paste::new, paste::update, paste::list, 
                                 paste::delete_sessionless, paste::delete,
                                 paste::show])
        .mount("/image", routes![image::new, image::show,
                                 image::delete, image::delete_sessionless,
                                 image::list, image::full, image::thumb])
        .mount("/video", routes![video::new, video::show, video::delete,
                                 video::delete_sessionless, video::full, video::list])
        .mount("/file", routes![files::get, files::delete, files::delete_sessionless,
                                files::list, files::new])
        .mount("/manage", routes![manage::image, manage::video, manage::paste,
                                  manage::my_images, manage::my_images_pageless, 
                                  manage::my_videos, manage::my_videos_pageless,
                                  manage::my_files, manage::my_files_pageless,
                                  manage::my_pastes, manage::my_pastes_pageless,
                                  manage::request_auth_cookie, manage::request_auth_url,
                                  manage::base_redirect])
        .mount("/meta", routes![meta::get_version, meta::get_latest_session,
                                meta::get_latest])
        .mount("/static", routes![files::static_asset])
        //.mount("/admin", routes![jobs::list_jobs, jobs::job_status])
        .mount("/", routes![favicon, verify_ssl])
        .manage(self::dbtools::init_pool())
        .launch();
}

#[get("/.well-known/acme-challenge/yG798UxIE5q0BXO6xuf_yVQGx5GkXcURa7q0PKnmhk4")]
pub fn verify_ssl() -> String {
    String::from("yG798UxIE5q0BXO6xuf_yVQGx5GkXcURa7q0PKnmhk4.NIKSE7P9moyu47YXlGpDd8NMqKL_JJaev0YL5gWDX7A")
}

#[get("/favicon.ico")]
fn favicon() -> Option<NamedFile> {
    NamedFile::open(Path::new("favicon.ico")).ok()
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
