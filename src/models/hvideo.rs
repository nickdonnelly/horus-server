use chrono::NaiveDateTime;

use schema::horus_videos;

#[derive(AsChangeset, Queryable, Serialize, Identifiable, Insertable)]
#[table_name = "horus_videos"]
pub struct HVideo
{
    pub id: String,
    pub title: Option<String>,
    pub owner: i32,
    pub filepath: String,
    pub date_added: NaiveDateTime,
    pub is_expiry: bool,
    pub expiration_time: Option<NaiveDateTime>,
}
