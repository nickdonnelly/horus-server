use std::panic;

use rocket::{self, http::{ContentType, Header, Status}, local::Client};
use diesel::connection::SimpleConnection;

use horus_server::{self, routes::paste::*};
use test::{run_test, sql::*};

#[test]
fn does_show()
{
    run(|| {
        let client = get_client();
        let req = client.get(String::from("/") + PASTE_ID);
        let mut response = req.dispatch();

        assert_eq!(
            response.content_type(),
            Some(ContentType::HTML),
            "Bad content-type. Expected HTML, got {:?}",
            response.content_type()
        );
        assert_eq!(
            response.status(),
            Status::Ok,
            "Bad response status, expected 200 OK, got {}",
            response.status()
        );
        assert!(
            response.body_string().unwrap().contains(PASTE_DATA),
            "Body did not contain paste data."
        );
    });
}

#[test]
fn does_list()
{
    run(|| {
        let client = get_client();
        let mut url = String::from("/");
        url.push_str(USER_ID.to_string().as_str());
        url.push_str("/list/0");
        let req = client.get(url.as_str()).header(api_key_header());
        let mut response = req.dispatch();

        assert_eq!(
            response.status(),
            Status::Ok,
            "Bad response status, expected 200 OK, got {}",
            response.status()
        );

        assert_eq!(
            response.content_type(),
            Some(ContentType::JSON),
            "Bad content-type. Expected JSON, got {:?}",
            response.content_type()
        );

        let res = response.body_string().unwrap();
        assert!(
            res.contains(PASTE_DATA),
            "Couldn't find {} in JSON response. Response was: \n{}",
            PASTE_DATA,
            res
        );
    });
}

#[test]
fn creates_new()
{
    run(|| {
        let body = r#"{"is_expiry":false, "paste_data":"test_paste","title":"Example Title"}"#;
        let client = get_client();
        let req = client
            .post("/new")
            .header(api_key_header())
            .header(Header::new("content-type", "application/json"))
            .body(body);
        let res = req.dispatch();

        assert_eq!(res.status(), Status::Created);

        let loc = res.headers().get_one("location").unwrap();
        let id = loc.replace("/paste/", "");
        assert_eq!(
            loc,
            String::from("/paste/") + &id,
            "Got unexpected body: {}",
            body
        );

        // Now run a get on it
        let req = client.get(String::from("/") + &id);
        let mut response = req.dispatch();

        assert_eq!(
            response.content_type(),
            Some(ContentType::HTML),
            "Bad content-type. Expected HTML, got {:?}",
            response.content_type()
        );
        assert_eq!(
            response.status(),
            Status::Ok,
            "Bad response status, expected 200 OK, got {}",
            response.status()
        );
        let body = response.body_string().unwrap();
        assert!(
            body.contains("test_paste"),
            "Body did not contain paste data."
        );
        assert!(body.contains("Example Title"));
    });
}

#[test]
fn deletes_correctly()
{
    run(|| {
        let client = get_client();
        let req = client
            .delete(String::from("/") + PASTE_ID)
            .header(api_key_header());
        let res = req.dispatch();

        assert_eq!(
            res.status(),
            Status::Ok,
            "Got bad status: expected 200 OK, got {}",
            res.status()
        );

        let req = client.get(String::from("/") + PASTE_ID);
        let res = req.dispatch();

        assert_eq!(
            res.status(),
            Status::NotFound,
            "Got bad status:: expected 404 NOT FOUND, got {}",
            res.status()
        );
    });
}

#[test]
fn updates_correctly()
{
    run(|| {
        let body = format!(
            r#"{{"id":"{id}", "paste_data": "new_data", "duration_type": "days", "duration_val": -1}}"#,
            id = PASTE_ID);
        let client = get_client();
        let req = client
            .put(String::from("/") + PASTE_ID)
            .header(api_key_header())
            .header(Header::new("content-type", "application/json"))
            .body(body);
        let response = req.dispatch();

        assert_eq!(response.status(), Status::Accepted);

        let req = client.get(String::from("/") + PASTE_ID);
        let mut response = req.dispatch();

        assert_eq!(response.status(), Status::Ok);
        assert!(response.body_string().unwrap().contains("new_data"));
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
    setup_sql.push_str(sql_insert_session().as_str());
    setup_sql.push_str(sql_insert_license().as_str());
    setup_sql.push_str(sql_insert_paste().as_str());

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
        .mount("/", routes![show, list, new, delete, update])
        .manage(horus_server::dbtools::init_pool());

    Client::new(rocket).expect("valid rocket instance")
}
