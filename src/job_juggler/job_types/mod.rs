use super::JobResult;

trait ExecutableJob {
    fn execute(self) -> JobResult;
}
