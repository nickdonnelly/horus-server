use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::pg::PgConnection;

use schema::horus_files;
use models::traits::passwordable;

#[derive(Queryable, Serialize, Identifiable, Insertable, AsChangeset)]
#[table_name = "horus_files"]
pub struct HFile
{
    pub id: String,
    pub owner: i32,
    pub filename: String,
    pub filepath: String,
    pub date_added: NaiveDateTime,
    pub is_expiry: bool,
    pub expiration_time: Option<NaiveDateTime>,
    pub download_counter: Option<i32>,
    pub password: Option<String>
}

impl passwordable::Passwordable for HFile {
    fn set_password(&mut self, password: Option<String>, conn: &PgConnection) -> Option<String>
    {
        use diesel::SaveChangesDsl;
        self.password = passwordable::retrieve_hashed(password);
        let result = self.save_changes::<HFile>(conn);
        match result {
            Ok(_) => None,
            Err(e) => Some(format!("{}", e))
        }
    }

    fn get_hashed_password(&self, conn: &PgConnection) -> Option<String>
    {
        use schema::horus_files::dsl::*;
        horus_files.find(&self.id).select(password).get_result::<Option<String>>(conn).unwrap()
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
