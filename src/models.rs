use diesel::associations::Identifiable;
use super::schema::*;

#[derive(Queryable, Identifiable, Serialize, Deserialize)]
#[table_name="horus_users"]
pub struct User {
    pub id: i32,
    pub first_name: String,
    pub last_name: Option<String>,
    pub email: String,
}

#[derive(AsChangeset, Deserialize)]
#[table_name="horus_users"]
pub struct UserForm {
    first_name: Option<String>,
    last_name: Option<String>,
    email: Option<String>,
}

#[derive(Queryable)]
pub struct LicenseKey {
    pub license_key: String,
    pub issued_to: u32,
    pub issued_on: String,
    pub valid_until: String,
    pub rate_limit: u32,
}