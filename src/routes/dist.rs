use bcrypt::{hash, verify, DEFAULT_COST};
use diesel::{self, prelude::*};
use rand::{self, Rng};

use rocket::http::Status;
use rocket::data::Data;
use rocket::response::{status, Failure, Redirect};

use {dbtools, DbConn};
use schema::{self, deployment_keys::dsl::*};
use fields::{Authentication, PrivilegeLevel};
use models::{DeploymentKey, HorusVersion, JobPriority, LicenseKey, NewJob};
use models::job_structures::{self, Deployment};
use job_juggler;

#[get("/version")]
pub fn version_legacy() -> Redirect
{
    Redirect::to("/dist/version/win64")
}

#[get("/version/<plat>")]
/// If no platform is given, it is assumed to be win64 just for compatability with older versions.
pub fn get_version(plat: Option<String>, conn: DbConn) -> Result<String, status::Custom<String>>
{
    use schema::horus_versions::dsl::*;

    let plat = if plat.is_none() {
        "win64".to_string()
    } else {
        plat.unwrap()
    };

    let version = horus_versions
        .filter(platform.eq(plat))
        .order(deploy_timestamp.desc())
        .first::<HorusVersion>(&*conn);

    if version.is_err() {
        return Err(status::Custom(
            Status::BadRequest,
            "Couldn't fetch latest version. Are you sure your platform is correct?".to_string(),
        ));
    }

    let version = version.unwrap();
    Ok(version.version_string())
}

#[get("/latest/<plat>")]
pub fn get_latest(plat: String, conn: DbConn, _auth: Authentication) -> Result<Redirect, Failure>
{
    use schema::horus_versions::dsl::*;

    let ver = horus_versions
        .filter(platform.eq(plat))
        .order(deploy_timestamp.desc())
        .first::<HorusVersion>(&*conn);

    if ver.is_err() {
        return Err(Failure(Status::NotFound));
    }

    let ver = ver.unwrap();
    let url = dbtools::s3::get_s3_presigned_url(ver.aws_path());

    if url.is_err() {
        Err(Failure(Status::ServiceUnavailable))
    } else {
        let url = url.unwrap();
        Ok(Redirect::to(&url))
    }
}

#[post("/deploy/publish/<platform>/<version_s>")]
pub fn enable_deployment(
    version_s: String,
    platform: String,
    conn: DbConn,
    depkey: DeploymentKey,
) -> Result<status::Custom<()>, Failure>
{
    use schema::horus_versions;
    // Verify key is the one used to deploy originally.
    let dbobj = horus_versions::dsl::horus_versions
        .find((&platform, &version_s))
        .first(&*conn);

    if dbobj.is_err() {
        return Err(Failure(Status::NotFound));
    }

    let mut version: HorusVersion = dbobj.unwrap();

    if version.deployment_key_hash() != depkey.hash() {
        return Err(Failure(Status::Unauthorized));
    }

    // we are authed, make the change
    version.publish();
    let db_result = diesel::update(
        horus_versions::dsl::horus_versions.find((&platform, &version_s)),
    ).set(&version)
        .execute(&*conn);

    if db_result.is_err() {
        return Err(Failure(Status::Unauthorized));
    } else {
        db_result.unwrap();
    }

    Ok(status::Custom(Status::Ok, ()))
}

/// Returns HTTP created with an integer id for the deployment.
#[post("/deploy/new/<platform>/<version>", format = "application/octet-stream",
       data = "<update_package>")]
pub fn deploy(
    platform: String,
    version: String,
    update_package: Data,
    depkey: DeploymentKey, // encompasses license key
) -> Result<status::Custom<String>, Failure>
{
    use std::io::Read;

    // not more than xxx.xxx.xxx not less than x.x.x
    // TODO: Regex this.
    if version.len() > 11 || version.len() < 5 {
        return Err(Failure(Status::BadRequest));
    }

    let file_data: Vec<u8> = update_package.open().bytes().map(|x| x.unwrap()).collect();

    let deployment_data = Deployment::new(file_data, depkey.hash(), version, platform.clone());
    let deployment_data = job_structures::binarize(&deployment_data);

    // Create job with file data.
    let new_job = NewJob::new(
        depkey.get_owner(),
        "deployment:deploy:".to_string() + &platform,
        Some(deployment_data),
        JobPriority::System,
    );

    let queue_result = job_juggler::enqueue_job(new_job);

    if queue_result.is_err() {
        return Err(Failure(Status::InternalServerError));
    }

    queue_result.unwrap();

    Ok(status::Custom(
        Status::Accepted,
        "Job queued for processing.".to_string(),
    ))
}

/// Verifies if a key is correct and returns its database object if so.
pub fn verify_key(
    lkey: LicenseKey,
    depkey: DeploymentKey,
    deployment_key: String,
) -> Result<DeploymentKey, ()>
{
    let key_hash_valid = verify(&deployment_key, &depkey.hash()).unwrap();
    if !key_hash_valid {
        return Err(());
    }

    let conn = super::super::dbtools::get_db_conn_requestless().unwrap();
    let depkey_query: Result<DeploymentKey, _> = deployment_keys
        .filter(schema::deployment_keys::dsl::key.eq(&depkey.hash()))
        .first(&conn);

    if depkey_query.is_ok() {
        let depkey_query = depkey_query.unwrap();

        if depkey_query.license_key == lkey.key {
            Ok(depkey_query)
        } else {
            Err(())
        }
    } else {
        Err(())
    }
}

/// Issue a deployment key.
/// Returns a tuple containing a plaintext deployment key and its corresponding
/// database object.
pub fn issue_deployment_key(l_key: LicenseKey) -> Result<(String, DeploymentKey), String>
{
    if l_key.privilege_level < PrivilegeLevel::System as i16 {
        return Err("Privilege level of API key is too low to issue deployment key.".to_string());
    }

    let random_key: String = rand::thread_rng()
        .gen_ascii_chars()
        .take(128)
        .collect::<Vec<char>>()
        .iter()
        .fold(String::new(), |mut acc, c| {
            acc.push(*c);
            acc
        });

    let random_hash = hash(&random_key, DEFAULT_COST).unwrap();
    let result_key = DeploymentKey::new(random_hash, &l_key);

    let connection = super::super::dbtools::get_db_conn_requestless().unwrap();
    let dep_key_result = diesel::insert_into(schema::deployment_keys::table)
        .values(&result_key)
        .get_result::<DeploymentKey>(&connection);

    if dep_key_result.is_err() {
        return Err(format!("Database error {}.", dep_key_result.err().unwrap()));
    }

    let dep_key_result = dep_key_result.unwrap();

    // Return the actual db object, not the one we made
    Ok((String::from(random_key), dep_key_result))
}
