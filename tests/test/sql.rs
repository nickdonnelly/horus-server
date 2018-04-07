use rocket::http::Header;

// Constants for dummy data
pub const USER_ID: i32 = 999;
pub const USER_FNAME: &'static str = "test";
pub const USER_LNAME: &'static str = "user";
pub const USER_EMAIL: &'static str = "testuser@example.com";

#[cfg_attr(rustfmt, rustfmt_skip)]
pub const TOKEN_STR: &'static str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
#[cfg_attr(rustfmt, rustfmt_skip)]
pub const DEPKEY: &'static str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
pub const DEPKEY_HASH: &'static str =
    "$2y$12$QDrb7qbHfUhOL2PShTLHJe0VdFXRdnHcj3cJeBemDklzkTpyaw3Je";
pub const API_KEY: &'static str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

pub const PASTE_ID: &'static str = "abcdefg";
pub const PASTE_DATA: &'static str = "paste_data_123 paste data lalala";

pub const VIDEO_ID: &'static str = "defghij";
pub const VIDEO_PATH: &'static str = "/live/videos/defghij.webm";

pub const JOB_ID: i32 = 582;
pub const JOB_NAME: &'static str = "test_job";
pub const JOB_DATA: &'static str = "this_is_test_data";
pub const JOB_STATUS: &'static str = "1";

pub const IMAGE_ID: &'static str = "hijklm";
pub const IMAGE_PATH: &'static str = "/live/images/hijklm.png";

pub const FILE_ID: &'static str = "asdfgh";
pub const FILE_NAME: &'static str = "hijklm.txt";
pub const FILE_PATH: &'static str = "/live/files/hijklm.txt";

/// Helpers

/// Returns a header for test authentication
pub fn auth_header<'a>() -> Header<'a>
{
    Header::new("x-api-test", USER_ID.to_string() + "/0")
}

/// Returns a header for test api key.
pub fn api_key_header<'a>() -> Header<'a>
{
    Header::new("x-api-key", API_KEY)
}

/// Returns a header for a test deployment key
pub fn depkey_header<'a>() -> Header<'a>
{
    Header::new("x-deployment-key", DEPKEY)
}

/// Insertions
pub fn sql_insert_user() -> String
{
    format!(
        "INSERT INTO horus_users(id, first_name, last_name, email) values({}, '{}', '{}', '{}') \
         ON CONFLICT DO NOTHING;",
        USER_ID, USER_FNAME, USER_LNAME, USER_EMAIL
    )
}

/// Requires the calling of sql_insert_user first.
pub fn sql_insert_license() -> String
{
    format!(
        "INSERT INTO horus_license_keys(key, issued_on, valid_until) \
         values('{key}', now(), now() + interval '7 days') ON CONFLICT DO NOTHING; \
         INSERT INTO horus_licenses(key, owner) values('{key}', 999) ON CONFLICT DO NOTHING;",
        key = API_KEY
    )
}

/// requires the calling of sql_insert_user first.
pub fn sql_insert_depkey() -> String
{
    format!(
        "INSERT INTO deployment_keys(key, license_key) values('{}', '{}') ON CONFLICT DO NOTHING;",
        DEPKEY_HASH, API_KEY
    )
}

pub fn sql_insert_job() -> String
{
    format!(
        "INSERT INTO horus_jobs(id, owner, job_name, job_status, job_data) \
         values({}, {}, '{}', {}, '{}') \
         ON CONFLICT DO NOTHING;",
        JOB_ID, USER_ID, JOB_NAME, JOB_STATUS, JOB_DATA
    )
}

/// Requires the calling of sql_insert_user first.
pub fn sql_insert_session() -> String
{
    format!(
        "INSERT INTO session_tokens(uid, token) VALUES(999, '{}') ON CONFLICT DO NOTHING;",
        TOKEN_STR
    )
}

/// Requires the calling of sql_insert_user first.
pub fn sql_insert_paste() -> String
{
    format!(
        "INSERT INTO horus_pastes(id, paste_data, owner) \
         values('{}', '{}', {}) ON CONFLICT DO NOTHING;",
        PASTE_ID, PASTE_DATA, USER_ID
    )
}

/// Requires the calling of sql_insert_user first.
pub fn sql_insert_video() -> String
{
    format!(
        "INSERT INTO horus_videos(id, owner, filepath) \
         values ('{}', {}, '{}') ON CONFLICT DO NOTHING;",
        VIDEO_ID, USER_ID, VIDEO_PATH
    )
}

/// Requires the calling of sql_insert_user first.
pub fn sql_insert_image() -> String
{
    format!(
        "INSERT INTO horus_images(id, owner, filepath) \
         values ('{}', {}, '{}') ON CONFLICT DO NOTHING;",
        IMAGE_ID, USER_ID, IMAGE_PATH
    )
}

/// Requires the calling of sql_insert_user first.
pub fn sql_insert_file() -> String
{
    format!(
        "INSERT INTO horus_files(id, owner, filename, filepath) \
         values ('{}', {}, '{}', '{}') ON CONFLICT DO NOTHING;",
        FILE_ID, USER_ID, FILE_NAME, FILE_PATH
    )
}

/// Deletions

pub fn sql_delete_user() -> String
{
    format!("DELETE FROM horus_users WHERE id = {};", USER_ID)
}

pub fn sql_delete_license() -> String
{
    format!(
        "DELETE FROM horus_licenses WHERE key = '{key}'; \
         DELETE FROM horus_license_keys WHERE key = '{key}';",
        key = API_KEY
    )
}

pub fn sql_delete_session() -> String
{
    format!("DELETE FROM session_tokens WHERE uid = {};", USER_ID)
}

pub fn sql_delete_paste() -> String
{
    format!("DELETE FROM horus_pastes WHERE owner = {};", USER_ID)
}
