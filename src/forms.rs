extern crate chrono;

use super::schema::*;
use self::chrono::NaiveDateTime;

#[derive(AsChangeset, Deserialize)]
#[table_name="horus_users"]
pub struct UserForm {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
}

#[derive(Deserialize)]
pub struct HPasteForm {
    pub title: Option<String>,
    pub paste_data: String,
    pub owner: i32,
    pub is_expiry: bool,
    pub expiration_time: Option<NaiveDateTime>,
}
