#![feature(plugin)]
#![plugin(rocket_codegen)]
extern crate diesel;
extern crate horus_server;
#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
extern crate serde_json;

mod endpoints;
mod test;
