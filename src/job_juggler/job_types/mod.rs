use super::JobResult;
use ::DbConn;

trait ExecutableJob {
    type DeserializeType;

    fn execute(self, conn: DbConn) -> JobResult;
}
