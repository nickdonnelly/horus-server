use ::job_juggler::JobResult;
use diesel::pg::PgConnection;

pub trait LoggableJob {
    fn log(&mut self, s: &str);
}

pub trait ExecutableJob {
    fn execute(self, conn: &PgConnection) -> JobResult;
}
