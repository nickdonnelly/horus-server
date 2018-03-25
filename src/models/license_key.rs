use diesel::prelude::*;
use chrono::NaiveDate;

use schema::horus_license_keys;
use models::License;
use dbtools;

#[derive(Insertable, Queryable, Serialize)]
#[table_name = "horus_license_keys"]
pub struct LicenseKey
{
    pub key: String,
    pub privilege_level: i16,
    pub issued_on: NaiveDate, // DO NOT MEASURE TIME
    pub valid_until: NaiveDate, // WITH THESE VALUES! NOT MONOTONIC!
                              //pub rate_limit: u32,
}

impl LicenseKey
{
    pub fn belongs_to(&self, uid: i32) -> bool
    {
        use ::schema::horus_licenses::dsl::*;

        let conn = dbtools::get_db_conn_requestless().unwrap();
        let license = horus_licenses
            .filter(key.eq(&self.key))
            .first::<License>(&conn);

        if license.is_err() {
            return false;
        }

        let license = license.unwrap();

        return license.owner == uid;
    }

    /// This function assumes the LicenseKey object is valid and in the db.
    pub fn get_owner(&self) -> i32
    {
        use ::schema::horus_licenses::dsl::*;
        let conn = dbtools::get_db_conn_requestless().unwrap();
        let license = horus_licenses
            .filter(key.eq(&self.key))
            .first::<License>(&conn)
            .unwrap();
        license.owner
    }
}


