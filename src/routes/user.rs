use diesel::{self, prelude::*};
use rocket::response::Failure;
use rocket::response::status;
use rocket::http::Status;
use rocket_contrib::Json;

use DbConn;
use models::{LicenseKey, PublicUser, SessionToken, User};
use forms::UserForm;
use schema::horus_users::dsl::*;

// Option usage allows us to automatically 404 if the record is not found
// by just returning "None".
#[get("/<uid>", rank = 2)]
pub fn show(uid: i32, conn: DbConn) -> Option<Json<PublicUser>>
{
    let user = horus_users.find(uid).first::<User>(&*conn);

    if user.is_err() {
        return None;
    }

    Some(Json(user.unwrap().without_sensitive_attributes()))
}

#[get("/<uid>", rank = 1)]
pub fn show_privileged(uid: i32, sess: SessionToken, conn: DbConn) -> Option<Json<User>>
{
    let user = horus_users.find(uid).first::<User>(&*conn);

    if user.is_err() {
        return None;
    }

    if sess.uid == uid {
        Some(Json(user.unwrap()))
    } else {
        None
    }
}

#[put("/<uid>", format = "application/json", data = "<updated_values>")]
pub fn update(
    uid: i32,
    apikey: LicenseKey,
    updated_values: Json<UserForm>,
    conn: DbConn,
) -> Result<status::Accepted<()>, Failure>
{
    if !apikey.belongs_to(uid) {
        return Err(Failure(Status::Unauthorized));
    }

    let user = updated_values.into_inner();

    let result = diesel::update(horus_users.filter(id.eq(uid)))
        .set(&user)
        .execute(&*conn);

    if result.is_err() {
        return Err(Failure(Status::NotFound));
    }

    Ok(status::Accepted(None))
}

#[delete("/<uid>")]
pub fn delete(uid: i32, apikey: LicenseKey, conn: DbConn) -> Result<status::Custom<()>, Failure>
{
    if !apikey.belongs_to(uid) {
        return Err(Failure(Status::Unauthorized));
    }

    let result = diesel::delete(horus_users.filter(id.eq(uid))).execute(&*conn);

    if result.is_err() {
        println!(
            "Database error while deleting user: {}",
            result.err().unwrap()
        );
        return Err(Failure(Status::InternalServerError));
    }

    Ok(status::Custom(Status::Ok, ()))
}

#[cfg(test)]
mod tests
{
    use std::panic;

    use rocket::{self, http::{Header, ContentType, Status}, local::Client};
    use rocket_contrib::Json;
    use diesel::connection::SimpleConnection;
    use serde_json;

    use super::*;
    use ::models::{User, PublicUser};
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
        let conn = ::dbtools::get_db_conn_requestless().unwrap();
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
        let conn = ::dbtools::get_db_conn_requestless().unwrap();
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
            .manage(::dbtools::init_pool());

        Client::new(rocket).expect("valid rocket instance")
    }

}
