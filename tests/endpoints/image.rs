use std::panic;

use rocket::{self, http::{Header, Status}, local::Client};
use diesel::connection::SimpleConnection;

use horus_server::{self, routes::image::*};
use test::{run_test, sql::*};


const B64_IMG: &'static str = include_str!("b64_img_data.txt");

#[test]
fn show()
{
    run(||{
        let client = get_client();
        let req = client.get("/image/".to_string() + IMAGE_ID);
        let res = req.dispatch();

        assert_eq!(res.status(), Status::Ok);
    });
}

#[test]
fn new()
{
    run(||{
        let client = get_client();
        let req = client.post("/image/new")
            .header(auth_header())
            .header(Header::new("content-type", "image/png"))
            .body(B64_IMG);
        let response = req.dispatch();

        assert_eq!(response.status(), Status::Created);

        let loc = response.headers().get_one("location").unwrap();

        let req = client.get("/".to_string() + loc);
        let response = req.dispatch();

        assert_eq!(response.status(), Status::Ok);
    });
}

#[test]
fn new_titled_with_exp()
{
    run(||{
        let client = get_client();
        let req = client.post(format!("/image/new/{}/{}/{}", "test_title123", "days", 2))
            .header(auth_header())
            .header(Header::new("content-type", "image/png"))
            .body(B64_IMG);
        let response = req.dispatch();

        assert_eq!(response.status(), Status::Created);
        
        let loc = response.headers().get_one("location").unwrap();
        let req = client.get(loc);
        let mut response = req.dispatch();

        assert_eq!(response.status(), Status::Ok);
        assert!(response.body_string().unwrap().contains("test_title123"));
    });
}

#[test]
fn update()
{
    run(|| {
        let client = get_client();
        let changes = r#"{ "title": "newer_title", "duration_type": "hours", "duration_val": 1 }"#;
        let req = client.put("/image/".to_string() + IMAGE_ID)
            .header(auth_header())
            .header(Header::new("content-type", "application/json"))
            .body(changes);
        let response = req.dispatch();

        assert_eq!(response.status(), Status::Accepted);

        let req = client.get("/image/".to_string() + IMAGE_ID);
        let mut response = req.dispatch();
        
        assert_eq!(response.status(), Status::Ok);
        assert!(response.body_string().unwrap().contains("newer_title"));
    });
}

#[test]
fn delete()
{
    run(|| {
        let client = get_client();
        let req = client.delete("/image/".to_string() + IMAGE_ID)
            .header(auth_header());
        let response = req.dispatch();

        assert_eq!(response.status(), Status::Ok);

        let req = client.get("/image/".to_string() + IMAGE_ID);
        let response = req.dispatch();

        assert_eq!(response.status(), Status::NotFound);
    });
}

#[test]
fn list()
{
    run(|| {
        let client = get_client();
        let req = client.post("/image/new")
            .header(auth_header())
            .header(Header::new("content-type", "image/png"))
            .body(B64_IMG);
        let new_id = req.dispatch().headers().get_one("location").unwrap()
            .trim_left_matches("/image/").to_string();

        let req = client.get(format!("/image/{}/list/0", USER_ID))
            .header(auth_header());
        let mut response = req.dispatch();

        assert_eq!(response.status(), Status::Ok);
        let bs = response.body_string().unwrap();
        
        assert!(bs.contains(IMAGE_ID));
        assert!(bs.contains(&new_id));
    });
}

#[test]
fn delete_authless_fails()
{
    run(|| {
        let client = get_client();
        let req = client.delete("/image/".to_string() + IMAGE_ID);
        let response = req.dispatch();

        assert_eq!(response.status(), Status::Unauthorized);
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
    setup_sql.push_str(sql_insert_image().as_str());

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
            "/image",
            routes![show, list, new, new_exp, new_titled, delete, update],
        )
        .manage(horus_server::dbtools::init_pool());

    Client::new(rocket).expect("valid rocket instance")
}
