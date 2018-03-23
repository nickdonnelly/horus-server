#![feature(plugin)]
#![plugin(rocket_codegen)]
extern crate horus_server;
extern crate diesel;
#[macro_use] extern crate rocket;
#[macro_use] extern crate rocket_contrib;
extern crate serde_json;

mod endpoints;

mod test {
    use std::panic;

    pub fn run_test<T, U, V>(test: T, setup: U, teardown: V) -> ()
        where T: FnOnce() -> () + panic::UnwindSafe,
              U: FnOnce() -> (),
              V: FnOnce() -> ()

    {
        setup();

        let result = panic::catch_unwind(|| {
            test()
        });

        teardown();

        assert!(result.is_ok());
    }
}
