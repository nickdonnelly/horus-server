use std::panic;

use rocket::{self, http::{Header, Status}, local::Client};
use diesel::connection::SimpleConnection;

use horus_server::{self, routes::video::*};
use test::{run_test, sql::*};

#[test]
fn new_vanilla()
{
    run(|| {
        let client = get_client();
        let req = client
            .post("/new")
            .header(api_key_header())
            .header(Header::new("content-type", "video/webm"))
            .body(include_str!("b64_vid_data.txt").trim());
        let response = req.dispatch();

        assert_eq!(response.status(), Status::Created);

        let loc = response.headers().get_one("location").unwrap();

        assert!(loc.contains("/video/"));

        let req = client.get(loc.trim_left_matches("/video"));
        let response = req.dispatch();

        assert_eq!(response.status(), Status::Ok);
    });
}

#[test]
fn new_titled_with_exp()
{
    run(|| {
        let client = get_client();
        let req = client
            .post("/new/A%20Test%20Title/hours/1")
            .header(api_key_header())
            .header(Header::new("content-type", "video/webm"))
            .body(include_str!("b64_vid_data.txt").trim());
        let response = req.dispatch();

        assert_eq!(response.status(), Status::Created);

        let loc = response.headers().get_one("location").unwrap();

        assert!(loc.contains("/video/"));

        let req = client.get(loc.trim_left_matches("/video"));
        let mut response = req.dispatch();

        assert_eq!(response.status(), Status::Ok);
        assert!(response.body_string().unwrap().contains("A Test Title"));
    });
}

#[test]
fn deletes()
{
    run(|| {
        let client = get_client();
        let req = client
            .delete(String::from("/") + VIDEO_ID)
            .header(api_key_header());
        let response = req.dispatch();

        assert_eq!(response.status(), Status::Ok);

        let req = client
            .get(String::from("/") + VIDEO_ID)
            .header(api_key_header());
        let response = req.dispatch();

        assert_eq!(response.status(), Status::NotFound);
    });
}

#[test]
fn updates()
{
    run(|| {
        let client = get_client();
        let body = r#"{ "title":"newtitle", "duration_type":"hours", "duration_val":1 }"#;
        let req = client
            .put(String::from("/") + VIDEO_ID)
            .header(auth_header())
            .header(Header::new("content-type", "application/json"))
            .body(body);
        let response = req.dispatch();

        assert_eq!(response.status(), Status::Accepted);

        let req = client.get(String::from("/") + VIDEO_ID);
        let mut response = req.dispatch();

        assert_eq!(response.status(), Status::Ok);
        assert!(response.body_string().unwrap().contains("newtitle"));
    });
}

#[test]
fn new_rejects_bad_data()
{
    run(|| {
        let client = get_client();
        let req = client
            .post("/new")
            .header(api_key_header())
            .header(Header::new("content-type", "video/webm"))
            .body("not_base64_webm");
        let response = req.dispatch();

        assert_eq!(response.status(), Status::BadRequest);
    });
}

#[test]
fn test_show()
{
    run(||{
        let client = get_client();
        let req = client.get("/".to_string() + VIDEO_ID);
        let res = req.dispatch();

        assert_eq!(res.status(), Status::Ok);
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
    setup_sql.push_str(sql_insert_video().as_str());

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
            "/",
            routes![show, list, new, new_exp, new_titled, delete, update],
        )
        .manage(horus_server::dbtools::init_pool());

    Client::new(rocket).expect("valid rocket instance")
}
