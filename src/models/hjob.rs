use std::boxed::Box;
use std::error::Error;

use chrono::NaiveDateTime;
use diesel::pg::Pg;
use diesel::row::Row;
use diesel::sql_types::Integer;
use diesel::deserialize::FromSqlRow;

use schema::horus_jobs;
use super::User;

#[repr(i16)]
#[derive(Serialize, Deserialize)]
pub enum JobStatus
{
    Waiting = 0,
    Queued = 1,
    Failed = 2,
    Running = 3,
    Complete = 10,
}

pub enum JobPriority
{
    DoNotProcess = -1, // the job isn't done being inserted.
    Normal = 0,        // normal jobs made by users
    Elevated = 1,      // jobs that are time dependent but not crucial
    High = 2,          // very important jobs
    System = 3,  // system level jobs like clearing caches and things that need to be done next.
    GodMode = 4, // Forcible override - shouldn't be used lightly.
                 // TODO: Directly insert godmode jobs into job queue instead of database.
}

#[derive(PartialEq, Debug, Serialize, Deserialize, Identifiable, Queryable, Associations,
         Insertable, AsChangeset)]
#[table_name = "horus_jobs"]
#[belongs_to(User, foreign_key = "owner")]
pub struct HJob
{
    pub id: i32,
    pub owner: i32,
    pub job_status: i32,
    pub job_name: String,
    pub job_data: Option<Vec<u8>>,
    pub time_queued: NaiveDateTime,
    pub priority: i32,
    pub logs: Option<String>,
}

#[derive(Insertable)]
#[table_name = "horus_jobs"]
pub struct NewJob
{
    owner: i32,
    job_name: String,
    pub job_data: Option<Vec<u8>>,
    pub priority: i32,
}

impl NewJob
{
    pub fn new(uid: i32, name: String, data: Option<Vec<u8>>, priority: JobPriority) -> Self
    {
        NewJob {
            owner: uid,
            job_name: name,
            job_data: data,
            priority: priority as i32,
        }
    }

    /// Return an instance of self without the data (used for quick insert
    /// without the data being present). Also sets the priority to `JobPriority::DoNotProcess`.
    pub fn without_data(&self) -> NewJob
    {
        NewJob {
            owner: self.owner,
            job_name: self.job_name.clone(),
            job_data: None,
            priority: JobPriority::DoNotProcess as i32,
        }
    }
}

impl FromSqlRow<Integer, Pg> for JobStatus
{
    fn build_from_row<R: Row<Pg>>(row: &mut R) -> Result<Self, Box<Error + Send + Sync>>
    {
        match i16::build_from_row(row)? {
            0 => Ok(JobStatus::Waiting),
            1 => Ok(JobStatus::Queued),
            2 => Ok(JobStatus::Failed),
            10 => Ok(JobStatus::Complete),
            v => Err(format!("Received bad value for JobStatus: {}", v).into()),
        }
    }
}
