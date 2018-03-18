use bincode::{ self, serialize, deserialize };
use serde::{ Serialize, Deserialize };

mod deployment;

pub use self::deployment::Deployment;

/// Turn a struct into a serialized vector of bytes to be passed
/// as job_data to the database.
pub fn binarize<T: Serialize>(to_binarize: &T) ->  Vec<u8>
{
    let result: bincode::Result<Vec<u8>> = serialize(to_binarize);
    result.unwrap()
}

/// Turn a binary vector into a struct of type T. May fail.
pub fn debinarize<'a, T: Deserialize<'a>>(to_debinarize: &'a [u8]) -> T
{
    let deserialized: bincode::Result<T> = deserialize(to_debinarize);
    deserialized.unwrap()
}
