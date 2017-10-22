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
        .mount("/image", routes![image::new, image::new_exp, image::new_titled, image::show,
                                 image::update, image::delete, image::delete_sessionless,
                                 image::list, image::full, image::thumb])
        .mount("/video", routes![video::new, video::new_titled, video::show, video::delete,
                                 video::update, video::delete_sessionless, video::full, 
                                 video::list, video::new_exp])
        .mount("/file", routes![files::get, files::delete, files::delete_sessionless,
                                files::list, files::new, files::new_exp])
        .mount("/manage", routes![manage::image, manage::video, manage::paste,
                                  manage::my_images, manage::my_images_pageless, 
                                  manage::my_videos, manage::my_videos_pageless,
                                  manage::my_files, manage::my_files_pageless,
                                  manage::my_pastes, manage::my_pastes_pageless,
                                  manage::request_auth_cookie, manage::request_auth_url,
                                  manage::base_redirect])
        .mount("/meta", routes![meta::get_version, meta::get_latest_session,
                                meta::get_latest, meta::changelogs])
        .mount("/static", routes![files::static_asset])
        //.mount("/admin", routes![jobs::list_jobs, jobs::job_status])
        .mount("/", routes![favicon, verify_ssl])
        .manage(self::dbtools::init_pool())
        .launch();
}

#[get("/.well-known/acme-challenge/kRLJd-GWcR0gToTfnXS-Kvyn8DcK-U6Es--9uA6nGsk")]
pub fn verify_ssl() -> String {
    String::from("kRLJd-GWcR0gToTfnXS-Kvyn8DcK-U6Es--9uA6nGsk.zFScHQkirc75cfQ9qjihdABaD_u16l-THYgvENWR30k")
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
