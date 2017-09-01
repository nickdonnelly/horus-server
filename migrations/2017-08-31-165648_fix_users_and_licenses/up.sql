-- Your SQL goes here
DROP TABLE IF EXISTS horus_users;
DROP TABLE IF EXISTS horus_licenses;

CREATE TABLE horus_users (
    id SERIAL NOT NULL,
    first_name VARCHAR NOT NULL,
    last_name VARCHAR,
    email varchar(255) UNIQUE NOT NULL,
    PRIMARY KEY (id)
);

CREATE TABLE horus_licenses(
    license_key CHAR(32) UNIQUE NOT NULL,
    issued_to SERIAL REFERENCES horus_users(id),
    issued_on date NOT NULL,
    valid_until date NOT NULL,
    rate_limit int,
    PRIMARY KEY (license_key)
);

