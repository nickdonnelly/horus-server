use std::boxed::Box;

use diesel::{self, prelude::*};
use diesel::pg::PgConnection;

use job_juggler::{ExecutableJob, JobResult, LoggableJob};

#[derive(Serialize, Deserialize, LoggableJob)]
#[LogName = "log_data"]
pub struct CreateImageThumbnail {
    pub image_id: String,
    pub image_data: Vec<u8>,
    pub log_data: String
}

impl ExecutableJob for CreateImageThumbnail {
    fn execute(self, conn: &PgConnection) -> (Box<Self>, JobResult) {
        
        // We only need imagemagick for one command, so we run it directly.
    }
}

/// A job that can be executed
pub trait ExecutableJob
{
    /// Execute this job, returning the object itself at the end alongside the result.
    fn execute(self, conn: &PgConnection) -> (Box<Self>, JobResult);
}
