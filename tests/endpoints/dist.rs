use std::panic;

use rocket::{self, http::{ContentType, Header, Status, StatusClass}, local::Client};
use diesel::connection::SimpleConnection;

use horus_server::{self, routes::dist::*};
use test::{run_test, sql::*};

#[test]
fn legacy_redirect()
{
    run(|| {
        let client = get_client();
        let req = client.get("/dist/version");
        let response = req.dispatch();

        assert_eq!(response.status().class(), StatusClass::Redirection);
    });
}

#[test]
fn version()
{
    run(|| {
        let client = get_client();
        let req = client.get("/dist/version/win64");
        let response = req.dispatch();
        assert_eq!(response.status(), Status::Ok);

        let req = client.get("/dist/version/linux");
        let response = req.dispatch();
        assert_eq!(response.status(), Status::Ok);
    });
}

#[test]
fn version_can_fail()
{
    run(|| {
        let client = get_client();
        let req = client.get("/dist/version/notaversion");
        let mut response = req.dispatch();

        assert_eq!(response.status(), Status::BadRequest);
        assert!(
            response
                .body_string()
                .unwrap()
                .contains("platform is correct?")
        );
    });
}

#[test]
fn get_latest()
{
    run(|| {
        let client = get_client();
        let req = client.get("/dist/latest/linux").header(auth_header());
        let response = req.dispatch();

        assert_eq!(response.status().class(), StatusClass::Redirection);
    });
}

#[test]
fn get_latest_can_fail()
{
    run(|| {
        let client = get_client();
        let req = client.get("/dist/latest/failure").header(auth_header());
        let response = req.dispatch();

        assert_eq!(response.status(), Status::NotFound);
    });
}

#[test]
fn deploy()
{
    run(|| {
        let client = get_client();
        let body = "test_body";
        let req = client
            .post("/dist/deploy/new/linux/99.99.99")
            .header(Header::new("content-type", "application/octet-stream"))
            .header(api_key_header())
            .header(depkey_header())
            .body(body);
        let mut response = req.dispatch();

        assert_eq!(response.status(), Status::Accepted);
        assert!(
            response
                .body_string()
                .unwrap()
                .contains("queued for processing.")
        );
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
    setup_sql.push_str(sql_insert_depkey().as_str());

    conn.batch_execute(&setup_sql).unwrap();
}

fn unsetup_db()
{
    let conn = horus_server::dbtools::get_db_conn_requestless().unwrap();

    let mut unsetup_sql = format!("DELETE FROM horus_jobs WHERE owner = {};", USER_ID);
    unsetup_sql.push_str(sql_delete_user().as_str());

    conn.batch_execute(&unsetup_sql).unwrap();
}

fn get_client() -> Client
{
    use rocket_contrib::templates::Template;
    let rocket = rocket::ignite()
        .attach(Template::fairing())
        .mount(
            "/dist",
            routes![
                version_legacy,
                get_version,
                get_latest,
                enable_deployment,
                deploy
            ],
        )
        .manage(horus_server::dbtools::init_pool());

    Client::new(rocket).expect("valid rocket instance")
}
