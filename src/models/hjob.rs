use std::boxed::Box;
use std::error::Error;

use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::pg::Pg;
use diesel::row::Row;
use diesel::sql_types::Integer;
use diesel::deserialize::FromSqlRow;

use super::super::schema::horus_jobs;

#[repr(i16)]
pub enum JobStatus {
    Waiting = 0,
    Queued = 1,
    Failed = 2,
    Complete = 10
}

#[derive(Identifiable, Queryable, Insertable, AsChangeset)]
#[table_name="horus_jobs"]
pub struct HJob {
    pub id: i32,
    pub owner: i32,
    pub job_status: i32,
    pub job_name: String,
    pub job_data: String,
    pub time_queued: NaiveDateTime
}

impl FromSqlRow<Integer, Pg> for JobStatus {
    fn build_from_row<R: Row<Pg>>(row: &mut R) -> Result<Self, Box<Error+Send+Sync>> {
        match i16::build_from_row(row)? {
            0 => Ok(JobStatus::Waiting),
            1 => Ok(JobStatus::Queued),
            2 => Ok(JobStatus::Failed),
            10 => Ok(JobStatus::Complete),
            v => Err(format!("Received bad value for JobStatus: {}", v).into())
        }
    }
}
