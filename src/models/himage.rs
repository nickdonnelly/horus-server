use chrono::NaiveDateTime;

use schema::horus_images;

#[derive(AsChangeset, Queryable, Serialize, Identifiable, Insertable)]
#[table_name = "horus_images"]
pub struct HImage
{
    pub id: String,
    pub title: Option<String>,
    pub owner: i32,
    pub filepath: String,
    pub date_added: NaiveDateTime,
    pub is_expiry: bool,
    pub expiration_time: Option<NaiveDateTime>,
}
