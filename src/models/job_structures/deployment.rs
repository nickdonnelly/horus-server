use std::boxed::Box;

use job_juggler::{ExecutableJob, JobResult, LoggableJob};

use diesel::{ self, prelude::* };
use diesel::pg::PgConnection;

#[derive(Serialize, Deserialize, LoggableJob)]
#[LogName = "log_data"]
pub struct Deployment {
    pub deployment_package: Vec<u8>,
    pub deployment_key_hash: String,
    pub version_string: String,
    pub platform_string: String,
    pub log_data: String,
}

impl Deployment {
    pub fn new(package: Vec<u8>, dkey_hash: String, s_version: String, s_platform: String) -> Self {
        Deployment {
            deployment_package: package,
            deployment_key_hash: dkey_hash,
            version_string: s_version,
            platform_string: s_platform,
            log_data: String::new(),
        }
    }
}

impl ExecutableJob for Deployment {
    fn execute(mut self, conn: &PgConnection) -> (Box<Self>, JobResult) {
        use dbtools;
        use models::{HorusVersion, NewHorusVersion};

        let mut tl: String;

        // Get the filename and path details
        let s3_fname = &self.platform_string.clone();
        let s3_path = dbtools::get_path_deployment(&self.version_string, s3_fname);

        // Send it to s3
        tl = format!(
            "Sending package version {} for {} to S3",
            &self.version_string, &self.platform_string
        );
        &self.log(&tl);

        let s3_result =
            dbtools::private_resource_to_s3_named(&s3_fname, &s3_path, &self.deployment_package);

        if s3_result.is_err() {
            &self.log("Couldn't send data to S3...aborting deployment.");
            return (Box::new(self), JobResult::Failed);
        }

        &self.log("Done...result was ok.");
        &self.log("Inserting to database...");

        // fixed bug with older entried
        if self.platform_string == "windows" {
            self.platform_string = "win64".to_string();
        }

        let hversion = NewHorusVersion::new(
            self.deployment_key_hash.clone(),
            s3_path,
            self.version_string.clone(),
            self.platform_string.clone(),
        );

        let db_result = diesel::insert_into(::schema::horus_versions::table)
            .values(&hversion)
            .get_result::<HorusVersion>(conn);

        if db_result.is_err() {
            tl = format!("{}", db_result.err().unwrap());
            &self.log(&tl);
            &self.log("Couldn't insert into database...aborting deployment.");
            (Box::new(self), JobResult::Failed)
        } else {
            &self.log("Successfully inserted into database.");
            tl = format!(
                "Deployment of version {} for platform {} complete.",
                self.version_string, self.platform_string
            );
            &self.log(&tl);
            (Box::new(self), JobResult::Complete)
        }
    }
}
