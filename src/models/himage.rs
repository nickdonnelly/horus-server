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

#[derive(Serialize)]
pub struct FixedDateHImage
{
    pub id: String,
    pub title: Option<String>,
    pub owner: i32,
    pub filepath: String,
    pub date_added: String,
    pub is_expiry: bool,
    pub expiration_time: Option<NaiveDateTime>,
}

impl HImage {
    pub fn with_displayable_date(&self) -> FixedDateHImage
    {
        FixedDateHImage {
            id: (&self).id.clone(),
            title: (&self).title.clone(),
            owner: (&self).owner,
            filepath: (&self).filepath.clone(),
            date_added: format!("{}", &self.date_added.format("%d %b %Y\nat %H:%M")),
            is_expiry: (&self).is_expiry,
            expiration_time: (&self).expiration_time.clone()
        }
    }
}
