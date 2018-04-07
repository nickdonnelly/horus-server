use super::LicenseKey;
use schema::deployment_keys;

#[derive(Insertable, Queryable, Serialize, Deserialize, AsChangeset, Debug)]
#[table_name = "deployment_keys"]
pub struct DeploymentKey
{
    key: String,
    pub deployments: i32,
    pub license_key: String,
}

impl DeploymentKey
{
    pub fn new(key_hash: String, lkey: &LicenseKey) -> Self
    {
        Self {
            key: key_hash,
            license_key: lkey.key.clone(),
            deployments: 0,
        }
    }

    pub fn hash(&self) -> String
    {
        self.key.clone()
    }

    pub fn get_owner(&self) -> i32
    {
        use dbtools::get_db_conn_requestless;
        use models::License;
        use diesel::prelude::*;

        let conn = get_db_conn_requestless().unwrap();
        let license = ::schema::horus_licenses::dsl::horus_licenses
            .filter(::schema::horus_licenses::dsl::key.eq(&self.license_key))
            .first::<License>(&conn);

        match license {
            Err(e) => panic!("DeploymentKey::get_owner() error: {}", e),
            Ok(license) => return license.owner,
        }
    }
}
