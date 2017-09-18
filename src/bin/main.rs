#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate horus_server;
extern crate rocket;
extern crate rocket_contrib;

use horus_server::*;
use rocket_contrib::Template;

fn main() {
    use self::routes::*;
    check_dirs();
    rocket::ignite()
        .attach(Template::fairing())
        .mount("/user", routes![user::show, user::update])
        .mount("/key", routes![key::validity_check, key::issue])
        .mount("/paste", routes![paste::new, 
                                 paste::show, 
                                 paste::delete, 
                                 paste::list])
        .mount("/image", routes![image::new, 
                                 image::show, 
                                 image::delete, 
                                 image::list,
                                 image::full,
                                 image::thumb])
        .mount("/video", routes![video::new, 
                                 video::show, 
                                 video::delete, 
                                 video::list])
        .mount("/manage", routes![manage::my_images, manage::my_images_pageless, 
                                  manage::image,
                                  manage::my_videos, manage::my_videos_pageless,
                                  manage::video,
                                  manage::request_auth_cookie,
                                  manage::request_auth_url,])
        .mount("/static", routes![files::static_asset])
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
