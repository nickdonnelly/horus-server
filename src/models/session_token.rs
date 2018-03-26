use chrono::NaiveDateTime;

use schema::session_tokens;

#[derive(Identifiable, Insertable, Queryable, AsChangeset, Debug)]
#[primary_key(uid)]
#[table_name = "session_tokens"]
pub struct SessionToken
{
    pub uid: i32,
    pub token: String,
    pub use_count: Option<i32>,
    pub expires: Option<NaiveDateTime>,
    pub privilege_level: i32,
}
