use chrono::NaiveDateTime;

use schema::*;

#[derive(Deserialize, Serialize)]
//#[table_name="horus_images"]
pub struct HImageChangesetForm
{
    pub title: Option<String>,
    pub duration_type: String, // days,hours,minutes
    pub duration_val: isize,
}

#[derive(Deserialize)]
//#[table_name="horus_videos"]
pub struct HVideoChangesetForm
{
    pub title: Option<String>,
    pub duration_type: String, // days,hours,minutes
    pub duration_val: isize,
}

#[derive(Deserialize)]
//#[table_name="horus_pastes"]
pub struct HPasteChangesetForm
{
    pub title: Option<String>,
    pub paste_data: Option<String>,
    pub duration_type: String, // days,hours,minutes
    pub duration_val: isize,
}

#[derive(AsChangeset, Deserialize)]
#[table_name = "horus_users"]
pub struct UserForm
{
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
}

#[derive(Deserialize)]
pub struct HNewPasteForm
{
    pub title: Option<String>,
    pub paste_data: String,
    pub is_expiry: bool,
    pub expiration_time: Option<NaiveDateTime>,
}
