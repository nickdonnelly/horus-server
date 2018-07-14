use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::pg::PgConnection;

use schema::horus_videos;
use models::traits::passwordable;

#[derive(AsChangeset, Queryable, Serialize, Identifiable, Insertable)]
#[table_name = "horus_videos"]
pub struct HVideo
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

impl passwordable::Passwordable for HVideo {
    fn set_password(&mut self, password: Option<String>, conn: &PgConnection) -> Option<String>
    {
        use diesel::SaveChangesDsl;
        self.password = passwordable::retrieve_hashed(password);
        let result = self.save_changes::<HVideo>(conn);
        match result {
            Ok(_) => None,
            Err(e) => Some(format!("{}", e))
        }
    }

    fn get_hashed_password(&self, conn: &PgConnection) -> Option<String>
    {
        use schema::horus_videos::dsl::*;
        horus_videos.find(&self.id).select(password).get_result::<Option<String>>(conn).unwrap()
    }

    fn get_s3_location(&self) -> String
    {
        self.filepath.clone()
    }

    fn owner(&self) -> i32
    {
        self.owner
    }
}

