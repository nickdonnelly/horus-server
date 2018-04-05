use std::panic;

use rocket::{self, http::{ContentType, Header, Status}, local::Client};
use rocket_contrib::Json;
use diesel::connection::SimpleConnection;

use horus_server::{self, routes::manage::*};
use test::{run_test, sql::*};

#[test]
fn my_pastes()
{
    run(|| {
        let client = get_client();
        let req = client.get("/manage/pastes/0")
            .header(auth_header());
        let mut response = req.dispatch();

        assert_eq!(response.status(), Status::Ok);

        // TODO: Make sure its truncated (it's not currently.)
        assert!(response.body_string().unwrap().contains(PASTE_DATA));
    });
}

#[test]
fn my_images()
{
    run(||{
        let client = get_client();
        let req = client.get("/manage/images/0")
            .header(auth_header());
        let mut response = req.dispatch();

        assert_eq!(response.status(), Status::Ok);
        assert!(response.body_string().unwrap().contains(IMAGE_ID));
    });
}

#[test]
fn my_videos()
{
    run(||{
        let client = get_client();
        let req = client.get("/manage/videos/0")
            .header(auth_header());
        let mut response = req.dispatch();

        assert_eq!(response.status(), Status::Ok);
        assert!(response.body_string().unwrap().contains(VIDEO_ID));
    });
}

#[test]
fn my_files()
{
    run(||{
        let client = get_client();
        let req = client.get("/manage/files/0")
            .header(auth_header());
        let mut response = req.dispatch();

        assert_eq!(response.status(), Status::Ok);
        assert!(response.body_string().unwrap().contains(FILE_ID));
    });
}

#[test]
fn image() 
{
    run(||{
        let client = get_client();
        let req = client.get("/manage/image/".to_string() + IMAGE_ID)
            .header(auth_header());
        let mut response = req.dispatch();

        assert_eq!(response.status(), Status::Ok);
        assert!(response.body_string().unwrap().contains(IMAGE_ID));
    });
}

#[test]
fn video() 
{
    run(||{
        let client = get_client();
        let req = client.get("/manage/video/".to_string() + VIDEO_ID)
            .header(auth_header());
        let mut response = req.dispatch();

        assert_eq!(response.status(), Status::Ok);
        assert!(response.body_string().unwrap().contains(VIDEO_ID));
    });
}

#[test]
fn paste() 
{
    run(||{
        let client = get_client();
        let req = client.get("/manage/paste/".to_string() + PASTE_ID)
            .header(auth_header());
        let mut response = req.dispatch();

        assert_eq!(response.status(), Status::Ok);
        assert!(response.body_string().unwrap().contains(PASTE_ID));
    });
}

fn run<T>(test: T) -> ()
where
    T: FnOnce() -> () + panic::UnwindSafe,
{
    run_test(test, setup_db, unsetup_db);
}

fn setup_db()
{
    let conn = horus_server::dbtools::get_db_conn_requestless().unwrap();
    let mut setup_sql = String::new();

    setup_sql.push_str(sql_insert_user().as_str());
    setup_sql.push_str(sql_insert_license().as_str());

    setup_sql.push_str(sql_insert_paste().as_str());
    setup_sql.push_str(sql_insert_image().as_str());
    setup_sql.push_str(sql_insert_video().as_str());
    setup_sql.push_str(sql_insert_file().as_str());

    conn.batch_execute(&setup_sql).unwrap();
}

fn unsetup_db()
{
    let conn = horus_server::dbtools::get_db_conn_requestless().unwrap();
    // No need to delete everything, a user delete cascades.
    let unsetup_sql = sql_delete_user();

    conn.batch_execute(&unsetup_sql).unwrap();
}

fn get_client() -> Client
{
    use rocket_contrib::Template;
    let rocket = rocket::ignite()
        .attach(Template::fairing())
        .mount(
            "/manage",
            routes![my_images, my_files, my_videos, my_pastes, video, image, paste],
        )
        .manage(horus_server::dbtools::init_pool());

    Client::new(rocket).expect("valid rocket instance")
}
