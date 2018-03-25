use chrono::NaiveDateTime;

use schema::auth_tokens;

#[derive(Identifiable, Insertable, Queryable, AsChangeset)]
#[primary_key(uid)]
#[table_name = "auth_tokens"]
pub struct AuthToken
{
    pub uid: i32,
    pub token: String,
    pub expires: Option<NaiveDateTime>,
    pub privilege_level: i32,
}
