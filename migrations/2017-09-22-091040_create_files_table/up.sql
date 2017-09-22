-- Your SQL goes here
CREATE TABLE horus_files(
  id varchar(12) NOT NULL,
  owner integer NOT NULL REFERENCES horus_users(id),
  filename varchar(128) NOT NULL,
  filepath varchar(256) NOT NULL,
  date_added timestamp NOT NULL default now(),
  is_expiry boolean NOT NULL default false,
  expiration_time timestamp,
  PRIMARY KEY (id)
);
