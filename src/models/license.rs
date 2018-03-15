use super::super::schema::horus_licenses;

#[derive(Insertable, Queryable, Serialize)]
#[table_name="horus_licenses"]
pub struct License {
    pub key: String,
    pub owner: i32,
    pub type_: Option<i16>, // This way we still match "type", which is 
                            // otherwise a rust-reserved keyword.
}
