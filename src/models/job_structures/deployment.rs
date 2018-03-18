#[derive(Serialize, Deserialize)]
pub struct Deployment {
    pub deployment_package: Vec<u8>,
    pub deployment_key_hash: String,
    pub version_string: String,
    pub platform_string: String
}

impl Deployment {
    pub fn new(
        package: Vec<u8>,
        dkey_hash: String,
        version: String,
        platform: String)
        -> Self
    {
        Deployment {
            deployment_package: package,
            deployment_key_hash: dkey_hash,
            version_string: version,
            platform_string: platform
        }
    }
}
