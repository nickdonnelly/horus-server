use std::panic;

use rocket::{self, http::{ContentType, Header, Status}, local::Client};
use rocket_contrib::Json;
use diesel::connection::SimpleConnection;
use serde_json;

use horus_server::{self, models::{PublicUser, User}, routes::user::*};
use test::{run_test, sql::*};

#[test]
fn does_show()
{
    run(|| {
        let client = get_client();
        let req = client.get("/999");
        let mut response = req.dispatch();

        assert!(response.status() == Status::Ok, "Wrong status code!");
        assert_eq!(response.content_type(), Some(ContentType::JSON));

        let user: PublicUser = serde_json::from_str(&response.body_string().unwrap()).unwrap();

        assert_eq!(user.id, 999);
        assert_eq!(user.first_name, "test");
    });
}

#[test]
fn not_found_show()
{
    run(|| {
        let client = get_client();
        let req = client.get("/9999");
        let response = req.dispatch();

        assert!(response.status() == Status::NotFound);
    });
}

#[test]
fn does_show_privileged()
{
    run(||{
        let client = get_client();
        let req = client.get("/me")
            .header(auth_header());
        let mut response = req.dispatch();

        assert_eq!(response.status(), Status::Ok);

        let user: User = serde_json::from_str(&response.body_string().unwrap()).unwrap();

        assert_eq!(user.id, 999);
        assert_eq!(user.first_name, "test");
        assert_eq!(user.last_name, Some("user".to_string()));
        assert_eq!(user.email, "testuser@example.com");

    });
}

#[test]
fn does_update()
{
    run(|| {
        let client = get_client();
        let req = client
            .put("/999")
            .header(api_key_header())
            .header(Header::new("content-type", "application/json"))
            .body("{\"first_name\":\"test1\"}");
        let response = req.dispatch();

        assert_eq!(response.status(), Status::Accepted);
    });
}

#[test]
fn does_delete()
{
    run(|| {
        let client = get_client();
        let req = client
            .delete("/999")
            .header(api_key_header());
        let response = req.dispatch();

        assert_eq!(response.status(), Status::Ok);

        // Check for a 404
        let req = client.get("/999");
        let response = req.dispatch();

        assert_eq!(response.status(), Status::NotFound);
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
    let rocket = rocket::ignite()
        .mount("/", routes![show, show_privileged, update, delete])
        .manage(horus_server::dbtools::init_pool());

    Client::new(rocket).expect("valid rocket instance")
}
