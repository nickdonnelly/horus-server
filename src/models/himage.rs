use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::pg::PgConnection;

use schema::horus_images;
use models::traits::passwordable;

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
    pub password: Option<String>
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
    pub password: Option<String>
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
            expiration_time: (&self).expiration_time.clone(),
            password: (&self).password.clone()
        }
    }

    pub fn new(id: String, 
       title: Option<String>, 
       owner: i32,
       filepath: String,
       date_added: NaiveDateTime,
       is_expiry: bool,
       expiration_time: Option<NaiveDateTime>)
        -> Self
    {
        HImage {
            id: id,
            title: title,
            owner: owner,
            filepath: filepath,
            date_added: date_added,
            is_expiry: is_expiry,
            expiration_time: expiration_time,
            password: None
        }
    }
}

impl passwordable::Passwordable for HImage {
    fn set_password(mut self, password: Option<String>, conn: &PgConnection) -> Option<String>
    {
        use diesel::SaveChangesDsl;
        self.password = passwordable::retrieve_hashed(password);
        let result = self.save_changes::<HImage>(conn);

        match result {
            Ok(_) => None,
            Err(e) => Some(format!("{}", e)),
        }
    }

    fn get_hashed_password(&self, conn: &PgConnection) -> Option<String>
    {
        use schema::horus_images::dsl::*;
        horus_images.find(&self.id).select(password).get_result::<Option<String>>(conn).unwrap()
    }

}

