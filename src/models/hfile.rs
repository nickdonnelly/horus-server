use chrono::NaiveDateTime;

use ::schema::horus_files;

#[derive(Queryable, Serialize, Identifiable, Insertable, AsChangeset)]
#[table_name = "horus_files"]
pub struct HFile {
    pub id: String,
    pub owner: i32,
    pub filename: String,
    pub filepath: String,
    pub date_added: NaiveDateTime,
    pub is_expiry: bool,
    pub expiration_time: Option<NaiveDateTime>,
    pub download_counter: Option<i32>,
}
