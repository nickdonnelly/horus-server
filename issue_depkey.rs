//! ```cargo
//! [dependencies]
//! horus_server = { path = "." }
//! diesel = "*"
//! ```
extern crate horus_server;
extern crate diesel;
use horus_server::routes::dist;
use horus_server::dbtools;
use diesel::prelude::*;
use horus_server::schema::horus_license_keys::dsl::*;

use std::env;

fn main() 
{
    println!("Starting.");
    let args: Vec<_> = env::args().collect();

    let _key = &args[args.len() - 1];
    print!("Obtaining database connection...");
    let conn = dbtools::get_db_conn_requestless().unwrap();
    print!("Obtained.\nRequesting license key...");

    let lkey = horus_license_keys.find(_key).first(&conn);
    print!("Done.\n");

    if lkey.is_err() {
        println!("Couldn't find that license key.");
        std::process::exit(-1);
    } 

    let lkey = lkey.unwrap();
    print!("Issuing key...");
    let dkey = dist::issue_deployment_key(lkey);
    print!("Done.\n");

    if dkey.is_err() {
        println!("Unable to issue deployment key due to a database error.");
        std::process::exit(-2);
    } else {
        let (kstr, _) = dkey.unwrap();
        println!("Issued key: {}", kstr);
    }
}

