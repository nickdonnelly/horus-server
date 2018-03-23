// Constants for dummy data
pub const USER_ID: i32 = 999;
pub const USER_FNAME: &'static str = "test";
pub const USER_LNAME: &'static str = "user";
pub const USER_EMAIL: &'static str = "testuser@example.com";

#[cfg_attr(rustfmt, rustfmt_skip)]
pub const TOKEN_STR: &'static str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
pub const API_KEY: &'static str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

/// Insertions

pub fn sql_insert_user() -> String
{
    format!("INSERT INTO horus_users(id, first_name, last_name, email) values({}, '{}', '{}', '{}') ON CONFLICT DO NOTHING;", USER_ID, USER_FNAME, USER_LNAME, USER_EMAIL)
}

/// Requires the calling of sql_insert_user first.
pub fn sql_insert_license() -> String
{
    format!("INSERT INTO horus_license_keys(key, issued_on, valid_until) values('{key}', now(), now() + interval '7 days') ON CONFLICT DO NOTHING; INSERT INTO horus_licenses(key, owner) values('{key}', 999) ON CONFLICT DO NOTHING;", key = API_KEY)
}

/// Requires the calling of sql_insert_user first.
pub fn sql_insert_session() -> String
{
    format!("INSERT INTO session_tokens(uid, token) VALUES(999, '{}') ON CONFLICT DO NOTHING;", TOKEN_STR)
}


/// Deletions

pub fn sql_delete_user() -> String
{
    format!("DELETE FROM horus_users WHERE id = {};", USER_ID)
}

pub fn sql_delete_license() -> String
{
    format!("DELETE FROM horus_licenses WHERE key = '{key}'; DELETE FROM horus_license_keys WHERE key = '{key}';", key = API_KEY)
}

pub fn sql_delete_session() -> String
{
    format!("DELETE FROM session_tokens WHERE uid = {};", USER_ID)
}
