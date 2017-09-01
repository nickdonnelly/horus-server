-- Your SQL goes here
CREATE TABLE horus_licenses(
    license_key CHAR(32) UNIQUE NOT NULL,
    issued_email VARCHAR(255) REFERENCES horus_users(email),
    issued_on date NOT NULL,
    valid_until date NOT NULL,
    rate_limit int,
    PRIMARY KEY (license_key)
);

CREATE TABLE horus_users (
    id SERIAL NOT NULL,
    first_name VARCHAR NOT NULL,
    last_name VARCHAR,
    email varchar(255) UNIQUE NOT NULL,
    license_key CHAR(32) REFERENCES horus_licenses(license_key) NOT NULL,
    PRIMARY KEY (id)
);