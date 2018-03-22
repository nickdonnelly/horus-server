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
    use diesel::{self, connection::SimpleConnection};

    use super::*;
    use ::test::run_test;

    // Exactly 128 characters.
    const TOKEN_STR: &'static str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    #[test]
    fn does_show()
    {
        run(|| {
            let client = get_client();
            let req = client.get("/999");
            let response = req.dispatch();

            assert!(response.status() == Status::Ok, "Wrong status code!");
            assert_eq!(response.content_type(), Some(ContentType::JSON));
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
        // TODO: Make a way to generate stub tokens that are valid for a single request
        // for testing purposes.
    }

    #[test]
    fn does_update()
    {

    }

    #[test]
    fn does_delete()
    {

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

        conn.batch_execute(&setup_sql).unwrap();
    }

    fn unsetup_db() 
    {
        let conn = ::dbtools::get_db_conn_requestless().unwrap();
        let mut unsetup_sql = String::from("DELETE FROM session_tokens WHERE uid = 999;");
        unsetup_sql.push_str("DELETE FROM horus_users WHERE id = 999;");

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
