table! {
    auth_tokens (uid) {
        uid -> Int4,
        token -> Bpchar,
        expires -> Nullable<Timestamp>,
        privilege_level -> Int4,
    }
}

table! {
    deployment_keys (key) {
        key -> Varchar,
        deployments -> Int4,
        license_key -> Bpchar,
    }
}

table! {
    horus_files (id) {
        id -> Varchar,
        owner -> Int4,
        filename -> Varchar,
        filepath -> Varchar,
        date_added -> Timestamp,
        is_expiry -> Bool,
        expiration_time -> Nullable<Timestamp>,
        download_counter -> Nullable<Int4>,
    }
}

table! {
    horus_images (id) {
        id -> Varchar,
        title -> Nullable<Varchar>,
        owner -> Int4,
        filepath -> Varchar,
        date_added -> Timestamp,
        is_expiry -> Bool,
        expiration_time -> Nullable<Timestamp>,
    }
}

table! {
    horus_jobs (id) {
        id -> Int4,
        owner -> Int4,
        job_status -> Int4,
        job_name -> Varchar,
        job_data -> Nullable<Bytea>,
        time_queued -> Timestamp,
        priority -> Int4,
        logs -> Nullable<Text>,
    }
}

table! {
    horus_license_keys (key) {
        key -> Bpchar,
        privilege_level -> Int2,
        issued_on -> Date,
        valid_until -> Date,
    }
}

table! {
    horus_licenses (key) {
        key -> Bpchar,
        owner -> Int4,
        #[sql_name = "type"]
        type_ -> Nullable<Int2>,
        resource_count -> Int4,
    }
}

table! {
    horus_pastes (id) {
        id -> Varchar,
        title -> Nullable<Varchar>,
        paste_data -> Text,
        owner -> Int4,
        date_added -> Timestamp,
        is_expiry -> Bool,
        expiration_time -> Nullable<Timestamp>,
    }
}

table! {
    horus_users (id) {
        id -> Int4,
        first_name -> Varchar,
        last_name -> Nullable<Varchar>,
        email -> Varchar,
    }
}

table! {
    horus_versions (platform, version_string) {
        id -> Int4,
        deployed_with -> Varchar,
        aws_bucket_path -> Varchar,
        version_string -> Varchar,
        platform -> Bpchar,
        deploy_timestamp -> Timestamp,
        is_public -> Bool,
    }
}

table! {
    horus_videos (id) {
        id -> Varchar,
        title -> Nullable<Varchar>,
        owner -> Int4,
        filepath -> Varchar,
        date_added -> Timestamp,
        is_expiry -> Bool,
        expiration_time -> Nullable<Timestamp>,
    }
}

table! {
    session_tokens (uid) {
        uid -> Int4,
        token -> Bpchar,
        use_count -> Nullable<Int4>,
        expires -> Nullable<Timestamp>,
        privilege_level -> Int4,
    }
}

joinable!(auth_tokens -> horus_users (uid));
joinable!(deployment_keys -> horus_license_keys (license_key));
joinable!(horus_files -> horus_users (owner));
joinable!(horus_images -> horus_users (owner));
joinable!(horus_jobs -> horus_users (owner));
joinable!(horus_licenses -> horus_license_keys (key));
joinable!(horus_licenses -> horus_users (owner));
joinable!(horus_pastes -> horus_users (owner));
joinable!(horus_versions -> deployment_keys (deployed_with));
joinable!(horus_videos -> horus_users (owner));
joinable!(session_tokens -> horus_users (uid));

allow_tables_to_appear_in_same_query!(
    auth_tokens,
    deployment_keys,
    horus_files,
    horus_images,
    horus_jobs,
    horus_license_keys,
    horus_licenses,
    horus_pastes,
    horus_users,
    horus_versions,
    horus_videos,
    session_tokens,
);
