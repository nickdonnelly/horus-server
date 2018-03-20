use super::super::schema::auth_tokens;
use chrono::NaiveDateTime;

#[derive(Identifiable, Insertable, Queryable, AsChangeset)]
#[primary_key(uid)]
#[table_name = "auth_tokens"]
pub struct AuthToken {
    pub uid: i32,
    pub token: String,
    pub expires: Option<NaiveDateTime>,
}
