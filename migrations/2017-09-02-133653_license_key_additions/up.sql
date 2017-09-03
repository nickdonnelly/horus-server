-- Your SQL goes here
DROP TABLE IF EXISTS horus_licenses;

CREATE TABLE horus_license_keys (
    key CHAR(32) UNIQUE NOT NULL,
    privilege_level SMALLINT DEFAULT 1,
    issued_on DATE NOT NULL,
    valid_until DATE NOT NULL,
    PRIMARY KEY (key)
);

CREATE TABLE horus_licenses (
    key CHAR(32) NOT NULL REFERENCES horus_license_keys(key),
    owner integer REFERENCES horus_users(id),
    type SMALLINT DEFAULT 1,
    PRIMARY KEY (key)
);