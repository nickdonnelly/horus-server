use ::job_juggler::JobResult;
use diesel::pg::PgConnection;

pub trait ExecutableJob {
    fn execute(self, conn: &PgConnection) -> JobResult;
}
