use ::job_juggler::JobResult;
use ::DbConn;

pub trait ExecutableJob {
    fn execute(self, conn: DbConn) -> JobResult;
}
