-- Your SQL goes here
ALTER TABLE horus_versions DROP CONSTRAINT horus_versions_pkey;
ALTER TABLE horus_versions ADD PRIMARY KEY (platform, version_string);
