use std::panic;

use rocket::{self, http::{Header, Status}, local::Client};
use rocket_contrib::Json;
use diesel::connection::SimpleConnection;
use serde_json;

use horus_server::{self, routes::jobs::*};
use test::{run_test, sql::*};

#[test]
pub fn job_status()
{
    run(||{
        let client = get_client();
        let req = client.get("/jobs/poll/".to_string() + &JOB_ID.to_string())
            .header(auth_header());
        let mut response = req.dispatch();

        assert_eq!(response.status(), Status::Ok);
        let bs = response.body_string().unwrap();
        assert!(bs.contains(&JOB_STATUS.to_string()))
    });
}

#[test]
pub fn job_status_not_exists()
{
    run(||{
        let client = get_client();
        let req = client.get("/jobs/poll/235235")
            .header(auth_header());
        let response = req.dispatch();

        assert_eq!(response.status(), Status::NotFound);
    });
}

#[test]
pub fn list_current_jobs()
{
    run(||{
        let client = get_client();
        let req = client.get("/jobs/active/".to_string() + &USER_ID.to_string())
            .header(auth_header());
        let mut response = req.dispatch();

        assert!(response.body_string().unwrap().contains(JOB_NAME));
    });
}

#[test]
pub fn list_all_jobs()
{
    run(||{
        let client = get_client();
        let req = client.get(format!("/jobs/all/{}/0", USER_ID))
            .header(auth_header());
        let mut response = req.dispatch();

        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.headers().get_one("content-type").unwrap(), "application/json");
        assert!(response.body_string().unwrap().contains(JOB_NAME));
    });
}

#[test]
pub fn denies_unauthed_user()
{
    run(|| {
        let client = get_client();
        let req = client.get("/jobs/all/123/0")
            .header(auth_header());
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
    setup_sql.push_str(sql_insert_job().as_str());

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
        .mount("/jobs", routes![retrieve_job_status, list_active_jobs, list_all_jobs])
        .manage(horus_server::dbtools::init_pool());

    Client::new(rocket).expect("valid rocket instance")
}
