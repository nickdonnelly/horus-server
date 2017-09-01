#[derive(Queryable)]
pub struct User {
    pub id: i32,
    pub first_name: String,
    pub last_name: Option<String>,
    pub email: String,
}

#[derive(Queryable)]
pub struct LicenseKey {
    pub license_key: String,
    pub issued_to: u32,
    pub issued_on: String,
    pub valid_until: String,
    pub rate_limit: u32,
}