use std::boxed::Box;

use job_juggler::JobResult;

use diesel::pg::PgConnection;

pub trait LoggableJob {
    fn log(&mut self, s: &str);
    fn logs(&self) -> String;
}

pub trait ExecutableJob {
    fn execute(self, conn: &PgConnection) -> (Box<Self>, JobResult);
}
