use std::panic;

use rocket::{self, http::{Header, Status}, local::Client};
use diesel::connection::SimpleConnection;
use serde_json;

use horus_server::{self, routes::key::*};
use test::{run_test, sql::*};

#[test]
fn issue()
{
    run(|| {
        let client = get_client();
        let req = client
            .post("/key/issue/999")
            .header(Header::new("x-api-test", USER_ID.to_string() + "/1")); // higher priv
        let mut response = req.dispatch();

        assert_eq!(response.status(), Status::Created);
        assert!(
            response
                .body_string()
                .unwrap()
                .contains(USER_ID.to_string().as_str())
        );
    });
}

#[test]
fn issue_fails()
{
    run(|| {
        let client = get_client();
        let req = client.post("/key/issue/999");
        let response = req.dispatch();

        assert_eq!(response.status(), Status::Unauthorized);
    });
}

#[test]
fn validity_check()
{
    run(|| {
        let client = get_client();
        let req = client.get(format!("/key/{}/validity-check", API_KEY));
        let mut response = req.dispatch();

        assert_eq!(response.status(), Status::Ok);
        assert!(response.body_string().unwrap().contains(API_KEY));
    });
}

#[test]
#[should_panic]
fn validity_check_fails()
{
    run(|| {
        let client = get_client();
        let req = client.get("/key/invalidkey/validity-check");
        let mut response = req.dispatch();

        assert_eq!(response.status(), Status::Ok);
        assert!(response.body_string().unwrap().contains(API_KEY));
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

    conn.batch_execute(&setup_sql).unwrap();
}

fn unsetup_db()
{
    let conn = horus_server::dbtools::get_db_conn_requestless().unwrap();

    let unsetup_sql = sql_delete_user();

    conn.batch_execute(&unsetup_sql).unwrap();
}

fn get_client() -> Client
{
    let rocket = rocket::ignite()
        .mount("/key", routes![validity_check, issue])
        .manage(horus_server::dbtools::init_pool());

    Client::new(rocket).expect("valid rocket instance")
}
