-- Your SQL goes here
CREATE TABLE horus_images(
  id varchar(12) UNIQUE NOT NULL,
  title varchar(128),
  filepath varchar(512) UNIQUE NOT NULL,
  date_added date NOT NULL,
  is_expiry bool NOT NULL DEFAULT false,
  expiration_time timestamp
);

CREATE TABLE horus_videos(
  id varchar(12) UNIQUE NOT NULL,
  title varchar(128),
  filepath varchar(512) UNIQUE NOT NULL,
  date_added date NOT NULL,
  is_expiry bool NOT NULL DEFAULT false,
  expiration_time timestamp
);

CREATE TABLE horus_pastes(
  id varchar(12) UNIQUE NOT NULL,
  title varchar(128),
  paste_data text,
  is_expiry bool NOT NULL DEFAULT false,
  expiration_time timestamp
);
