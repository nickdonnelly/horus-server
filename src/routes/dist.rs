extern crate rand;
extern crate bcrypt;
extern crate diesel;

use std;

use diesel::prelude::*;
use super::super::schema;
use super::super::schema::deployment_keys::dsl::*;
use self::bcrypt::{ DEFAULT_COST, hash, verify };
use self::rand::{ Rng, ThreadRng };

use super::super::models::{ LicenseKey, SessionToken, DeploymentKey };


/// Verifies if a key is correct and returns its database object if so.
fn verify_key(deployment_key: String) -> Result<DeploymentKey, ()>
{
    let key_hash = hash(&deployment_key, DEFAULT_COST).unwrap();
    let conn = super::super::dbtools::get_db_conn_requestless().unwrap();
    let depkey_query = deployment_keys.filter(
        schema::deployment_keys::dsl::key.eq(&key_hash)).first(&conn);

    if depkey_query.is_ok() {
        Ok(depkey_query.unwrap())
    } else {
        Err(())
    }
}

/// Issue a deployment key. 
/// Returns a tuple containing a plaintext deployment key and its corresponding
/// database object. 
pub fn issue_deployment_key(l_key: LicenseKey) -> Result<(String, DeploymentKey), ()>
{
    if l_key.privilege_level.is_none() {
        return Err(());
    }

    if l_key.privilege_level.unwrap() < 3 {
        return Err(());
    }

    let random_key: String = rand::thread_rng().gen_ascii_chars()
        .take(128).collect::<Vec<char>>().iter().fold(String::new(),
            | mut acc, c | {
                acc.push(*c);
                acc
            });
    
    let random_hash = hash(&random_key, DEFAULT_COST).unwrap();
    let result_key = DeploymentKey::new(random_hash);

    let connection = super::super::dbtools::get_db_conn_requestless().unwrap();
    let dep_key_result = diesel::insert_into(schema::deployment_keys::table)
        .values(&result_key).get_result::<DeploymentKey>(&connection);

    if dep_key_result.is_err() {
        return Err(());
    }

    let dep_key_result = dep_key_result.unwrap();
    
    // Return the actual db object, not the one we made
    Ok((String::from(random_key), dep_key_result)) 
}
