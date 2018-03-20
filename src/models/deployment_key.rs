use super::LicenseKey;
use super::super::schema::deployment_keys;

#[derive(Insertable, Queryable, Serialize, Deserialize, AsChangeset, Debug)]
#[table_name = "deployment_keys"]
pub struct DeploymentKey {
    key: String,
    pub deployments: i32,
    pub license_key: String,
}

impl DeploymentKey {
    pub fn new(key_hash: String, lkey: &LicenseKey) -> Self {
        Self {
            key: key_hash,
            license_key: lkey.key.clone(),
            deployments: 0,
        }
    }

    pub fn hash(&self) -> String {
        self.key.clone()
    }
}
