use chrono::NaiveDateTime;

use ::schema::horus_pastes;

#[derive(AsChangeset, Identifiable, Serialize, Insertable, Queryable, Deserialize)]
#[table_name = "horus_pastes"]
pub struct HPaste {
    pub id: String,
    pub title: Option<String>,
    pub paste_data: String,
    pub owner: i32,
    pub date_added: NaiveDateTime,
    pub is_expiry: bool,
    pub expiration_time: Option<NaiveDateTime>,
}
