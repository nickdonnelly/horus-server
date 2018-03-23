use std::panic;

use rocket::{self, http::{Header, ContentType, Status}, local::Client};
use rocket_contrib::Json;
use diesel::connection::SimpleConnection;
use serde_json;

use horus_server::{self, 
    models::{User, PublicUser},
    routes::user::*,
    };
use ::test::run_test;

// Exactly 128 characters.
const TOKEN_STR: &'static str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const API_KEY: &'static str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

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

/*  FIXME:
 *  This can't be used as a test until the latest version of rocket gains support for
 *  the private_cookie method. This is currently in 0.4.0-dev whereas we are currently
 *  on 0.3.6
#[test]
fn does_show_privileged()
{
    use rocket::http::Cookie;
    run(||{
        let client = get_client();
        let req = client.get("/999")
            .private_cookie(Cookie::new("horus_session", TOKEN_STR));
        let mut response = req.dispatch();

        assert_eq!(response.status(), Status::Ok);

        let user: User = serde_json::from_str(&response.body_string().unwrap()).unwrap();

        assert_eq!(user.id, 999);
        assert_eq!(user.first_name, "test");
        assert_eq!(user.last_name, Some("user".to_string()));
        assert_eq!(user.email, "testuser@example.com");

    });
}*/

#[test]
fn does_update()
{
    run(|| {
        let client = get_client();
        let req = client.put("/999")
            .header(Header::new("x-api-key", API_KEY))
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
        let req = client.delete("/999").header(Header::new("x-api-key", API_KEY));
        let response = req.dispatch();

        assert_eq!(response.status(), Status::Ok);

        // Check for a 404
        let req = client.get("/999");
        let response = req.dispatch();

        assert_eq!(response.status(), Status::NotFound);
    });
}

fn run<T>(test: T) -> ()
    where T: FnOnce() -> () + panic::UnwindSafe
{
    run_test(test, setup_db, unsetup_db);
}

fn setup_db()
{
    let conn = horus_server::dbtools::get_db_conn_requestless().unwrap();
    let mut setup_sql = String::from("INSERT INTO horus_users(id, first_name, last_name, email) \
        VALUES(999, 'test', 'user', 'testuser@example.com') ON CONFLICT DO NOTHING;");

    setup_sql.push_str("INSERT INTO session_tokens(uid, token) VALUES(999, '");
    setup_sql.push_str(TOKEN_STR);
    setup_sql.push_str("') ON CONFLICT DO NOTHING;");
    setup_sql.push_str("INSERT INTO horus_license_keys(key, issued_on, valid_until) values('");
    setup_sql.push_str(API_KEY);
    setup_sql.push_str("', now(), now() + interval '7 days') ON CONFLICT DO NOTHING;");
    setup_sql.push_str("INSERT INTO horus_licenses(key, owner) VALUES('");
    setup_sql.push_str(API_KEY);
    setup_sql.push_str("', 999) ON CONFLICT DO NOTHING;");


    conn.batch_execute(&setup_sql).unwrap();
}

fn unsetup_db() 
{
    let conn = horus_server::dbtools::get_db_conn_requestless().unwrap();
    let mut unsetup_sql = String::from("DELETE FROM session_tokens WHERE uid = 999;");
    unsetup_sql.push_str("DELETE FROM horus_licenses WHERE key = '");
    unsetup_sql.push_str(API_KEY);
    unsetup_sql.push_str("';");
    unsetup_sql.push_str("DELETE FROM horus_users WHERE id = 999;");
    unsetup_sql.push_str("DELETE FROM horus_license_keys WHERE key = '");
    unsetup_sql.push_str(API_KEY);
    unsetup_sql.push_str("';");

    conn.batch_execute(&unsetup_sql).unwrap();
}

fn get_client() -> Client
{
    let rocket = rocket::ignite()
        .mount("/", routes![show, show_privileged, update, delete])
        .manage(horus_server::dbtools::init_pool());

    Client::new(rocket).expect("valid rocket instance")
}
