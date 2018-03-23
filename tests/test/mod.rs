use std::panic;

pub mod sql;
pub use self::sql::*;

pub fn run_test<T, U, V>(test: T, setup: U, teardown: V) -> ()
where
    T: FnOnce() -> () + panic::UnwindSafe,
    U: FnOnce() -> (),
    V: FnOnce() -> (),
{
    setup();

    let result = panic::catch_unwind(|| test());

    teardown();

    assert!(result.is_ok());
}
