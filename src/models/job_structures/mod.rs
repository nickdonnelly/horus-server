use bincode::{ self, serialize, deserialize, Config };
use serde::{ Serialize, Deserialize };

mod deployment;

pub use self::deployment::Deployment;

/// Turn a struct into a serialized vector of bytes to be passed
/// as job_data to the database.
pub fn binarize<T: Serialize>(to_binarize: &T) ->  Vec<u8>
{
    let mut conf: Config = bincode::config();
    conf.no_limit();
    let result: bincode::Result<Vec<u8>> = conf.serialize(to_binarize);
    result.unwrap()
}

/// Turn a binary vector into a struct of type T. May fail.
pub fn debinarize<'a, T: Deserialize<'a>>(to_debinarize: &'a [u8]) -> Option<T>
{
    let mut conf: Config = bincode::config();
    conf.no_limit();
    let deserialized: bincode::Result<T> = conf.deserialize(to_debinarize);
    if deserialized.is_err() {
        println!("Deserialization error: {}", &deserialized.err().unwrap());
        None
    } else {
        Some(deserialized.unwrap())
    }
    //let r = deserialized.ok().unwrap();
    //println!("Unwrapped.");
    //r
}
