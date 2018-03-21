use ::schema::horus_users;

#[derive(Queryable, Identifiable, Associations, Insertable, Serialize, Deserialize)]
#[table_name = "horus_users"]
pub struct User {
    pub id: i32,
    pub first_name: String,
    pub last_name: Option<String>,
    pub email: String,
}
