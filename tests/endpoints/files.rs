use std::panic;

use rocket::{self, http::{Header, Status}, local::Client};
use diesel::connection::SimpleConnection;

use horus_server::{self, routes::files::*};
use test::{run_test, sql::*};


#[test]
fn get()
{
    run(|| {
        let client = get_client();
        let req = client.get("/file/".to_string() + FILE_ID);
        let mut response = req.dispatch();

        assert_eq!(response.status(), Status::Ok);
        assert!(response.body_string().unwrap().contains(FILE_NAME));
    });
}

#[test]
fn new()
{
    run(||{
        let client = get_client();
        let filecontent = "dummycontent";
        let new_fname = "filename123";
        let req = client.post("/file/new")
            .header(auth_header())
            .header(Header::new("content-type", "application/octet-stream"))
            .header(Header::new("content-disposition", new_fname))
            .body(filecontent);
        let response = req.dispatch();

        assert_eq!(response.status(), Status::Created);

        let loc = response.headers().get_one("location").unwrap();

        let mut response = client.get(loc).dispatch();
        assert!(response.body_string().unwrap().contains(new_fname));
    });
}

#[test]
fn new_exp()
{
    run(||{
        let client = get_client();
        let filecontent = "dummycontent";
        let new_fname = "filename123";
        let req = client.post("/file/new/hours/1")
            .header(auth_header())
            .header(Header::new("content-type", "application/octet-stream"))
            .header(Header::new("content-disposition", new_fname))
            .body(filecontent);
        let response = req.dispatch();

        assert_eq!(response.status(), Status::Created);

        let loc = response.headers().get_one("location").unwrap();

        let mut response = client.get(loc).dispatch();
        assert!(response.body_string().unwrap().contains(new_fname));
    });
}

#[test]
fn delete()
{
    run(|| {
        let client = get_client();
        let req = client.delete("/file/".to_string() + FILE_ID)
            .header(auth_header());
        let response = req.dispatch();

        assert_eq!(response.status(), Status::Ok);
        
        let response = client.get("/file/".to_string() + FILE_ID).dispatch();

        assert_eq!(response.status(), Status::NotFound);
    });
}

#[test]
fn list()
{
    run(||{
        let client = get_client();
        let req = client.post("/file/new")
            .header(auth_header())
            .header(Header::new("content-disposition", "fileabc"))
            .header(Header::new("content-type", "application/octet-stream"));
        let response = req.dispatch();

        assert_eq!(response.status(), Status::Created);

        let req = client.get(format!("/file/{}/list/0", USER_ID))
            .header(auth_header());
        let mut response = req.dispatch();

        assert_eq!(response.status(), Status::Ok);

        let bs = response.body_string().unwrap();
        
        assert!(bs.contains("fileabc"));
        assert!(bs.contains(FILE_NAME));
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
    setup_sql.push_str(sql_insert_file().as_str());

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
            "/file",
            routes![get, list, new, new_exp, delete],
        )
        .manage(horus_server::dbtools::init_pool());

    Client::new(rocket).expect("valid rocket instance")
}
