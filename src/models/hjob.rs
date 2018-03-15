use super::super::schema::horus_jobs;
use super::super::routes::jobs::JobStatus;
use chrono::NaiveDateTime;

#[derive(Identifiable, Queryable)]
#[table_name="horus_jobs"]
pub struct HJob {
    pub id: i32,
    pub owner: i32,
    pub job_status: JobStatus,
    pub job_data: String,
    time_queued: Option<NaiveDateTime>,
}
