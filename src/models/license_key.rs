use super::super::schema::horus_license_keys;
use chrono::NaiveDate;

#[derive(Insertable, Queryable, Serialize)]
#[table_name = "horus_license_keys"]
pub struct LicenseKey {
    pub key: String,
    pub privilege_level: Option<i16>,
    pub issued_on: NaiveDate, // DO NOT MEASURE TIME
    pub valid_until: NaiveDate, // WITH THESE VALUES! NOT MONOTONIC!
                              //pub rate_limit: u32,
}
