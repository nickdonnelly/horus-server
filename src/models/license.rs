use schema::horus_licenses;

#[derive(Insertable, Queryable, Serialize)]
#[table_name = "horus_licenses"]
pub struct License
{
    pub key: String,
    pub owner: i32,
    pub type_: Option<i16>, // Since "type" is a rust keyword, needed for diesel
    pub resource_count: i32
}
