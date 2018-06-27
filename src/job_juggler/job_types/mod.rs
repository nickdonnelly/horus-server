use std::boxed::Box;

use diesel::pg::PgConnection;

use job_juggler::JobResult;

/// A job that can produce logs to be stored in the database
pub trait LoggableJob
{
    /// Add a new line to the log.
    fn log(&mut self, s: &str);

    /// Get a copy of the logs.
    fn logs(&self) -> String;
}

/// A job that can be executed
pub trait ExecutableJob
{
    /// Execute this job, returning the object itself at the end alongside the result.
    fn execute(self, conn: &PgConnection) -> (Box<Self>, JobResult);
}
