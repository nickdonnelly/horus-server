use std::boxed::Box;

use diesel::pg::PgConnection;

use job_juggler::JobResult;

pub trait LoggableJob
{
    fn log(&mut self, s: &str);
    fn logs(&self) -> String;
}

pub trait ExecutableJob
{
    fn execute(self, conn: &PgConnection) -> (Box<Self>, JobResult);
}
