#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate horus_server;
extern crate rocket;
extern crate rocket_contrib;
#[macro_use] extern crate diesel;
#[macro_use] extern crate diesel_codegen;
extern crate r2d2_diesel;
extern crate r2d2;

use horus_server::*;
use self::models::*;
use diesel::prelude::*;
use diesel::pg::PgConnection;
use r2d2_diesel::ConnectionManager;
use rocket_contrib::Template;
use rocket::http::Status;
use rocket::request::{self, FromRequest};
use rocket::{Request, State, Outcome};

use std::ops::Deref;
use std::collections::HashMap;

// Database pooling definitions
type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;
static DATABASE_URL: &'static str = env!("DATABASE_URL");

// Global db connection
pub struct DbConn(pub r2d2::PooledConnection<ConnectionManager<PgConnection>>);

impl<'a, 'r> FromRequest<'a, 'r> for DbConn {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<DbConn, ()> {
        let pool = request.guard::<State<Pool>>()?;
        match pool.get() {
            Ok(conn) => Outcome::Success(DbConn(conn)),
            Err(_) => Outcome::Failure((Status::ServiceUnavailable, ()))
        }
    }
}

impl Deref for DbConn {
    type Target = PgConnection;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// Database pool initialization
fn init_pool() -> Pool {
    let config = r2d2::Config::default();
    let manager = ConnectionManager::<PgConnection>::new(DATABASE_URL);

    r2d2::Pool::new(config, manager).expect("db pool")
}

#[get("/<name>")]
fn index(name: String, conn: DbConn) -> Template {
    use horus_server::schema::horus_users::dsl::*;

    let mut context = HashMap::new();
    context.insert("name", name);

    let results = horus_users.find(1)
        .load::<User>(&*conn)
        .expect("Error! abc");
    
    for user in results {
        println!("email {}", user.email);
    }

    Template::render("index", &context)
}

fn main() {
    rocket::ignite()
        .attach(Template::fairing())
        .mount("/", routes![index])
        .manage(init_pool())
        .launch();
}
