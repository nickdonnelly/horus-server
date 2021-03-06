use chrono::NaiveDateTime;

use schema::horus_versions;

#[derive(Identifiable, Queryable, Serialize, Deserialize, AsChangeset)]
#[table_name = "horus_versions"]
#[primary_key(platform, version_string)]
pub struct HorusVersion
{
    id: i32,
    deployed_with: String, // hash
    aws_bucket_path: String,
    version_string: String,
    platform: String,
    deploy_timestamp: NaiveDateTime,
    is_public: bool,
}

/// This is what you use to insert new ones into the db (getting rid of forms soon)
#[derive(Insertable)]
#[table_name = "horus_versions"]
pub struct NewHorusVersion
{
    deployed_with: String,
    aws_bucket_path: String,
    version_string: String,
    platform: String,
}

impl NewHorusVersion
{
    pub fn new(
        deployment_key_hash: String,
        aws_path: String,
        version: String,
        platform: String,
    ) -> Self
    {
        NewHorusVersion {
            deployed_with: deployment_key_hash,
            aws_bucket_path: aws_path,
            version_string: version,
            platform: platform,
        }
    }
}

impl HorusVersion
{
    /// Returns the ID of the object or -1 if the object hasn't got one (not yet in database).
    pub fn id(&self) -> i32
    {
        self.id
    }

    pub fn version_string(&self) -> String
    {
        self.version_string.clone()
    }

    pub fn deployment_key_hash(&self) -> String
    {
        self.deployed_with.clone()
    }

    pub fn platform(&self) -> String
    {
        self.platform.clone()
    }

    pub fn aws_path(&self) -> String
    {
        self.aws_bucket_path.clone()
    }

    pub fn is_public(&self) -> bool
    {
        self.is_public
    }

    pub fn publish(&mut self)
    {
        self.is_public = true;
    }

    pub fn unpublish(&mut self)
    {
        self.is_public = false;
    }
}
