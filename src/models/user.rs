use schema::horus_users;

#[derive(Queryable, Identifiable, Associations, Insertable, Serialize, Deserialize)]
#[table_name = "horus_users"]
pub struct User
{
    pub id: i32,
    pub first_name: String,
    pub last_name: Option<String>,
    pub email: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PublicUser
{
    pub id: i32,
    pub first_name: String,
}

impl User
{
    pub fn without_sensitive_attributes(&self) -> PublicUser
    {
        PublicUser {
            id: self.id,
            first_name: self.first_name.clone(),
        }
    }
}
