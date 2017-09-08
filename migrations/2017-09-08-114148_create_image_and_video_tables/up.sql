-- Your SQL goes here
CREATE TABLE horus_images(
  id varchar(12) UNIQUE NOT NULL,
  title varchar(128),
  owner integer NOT NULL REFERENCES horus_users(id),
  filepath varchar(512) UNIQUE NOT NULL,
  date_added date NOT NULL,
  is_expiry bool NOT NULL DEFAULT false,
  expiration_time timestamp,
  PRIMARY KEY (id)
);

CREATE TABLE horus_videos(
  id varchar(12) UNIQUE NOT NULL,
  title varchar(128),
  owner integer NOT NULL REFERENCES horus_users(id),
  filepath varchar(512) UNIQUE NOT NULL,
  date_added date NOT NULL,
  is_expiry bool NOT NULL DEFAULT false,
  expiration_time timestamp,
  PRIMARY KEY (id)
);

CREATE TABLE horus_pastes(
  id varchar(12) UNIQUE NOT NULL,
  title varchar(128),
  paste_data text NOT NULL,
  owner integer NOT NULL REFERENCES horus_users(id),
  date_added date NOT NULL,
  is_expiry bool NOT NULL DEFAULT false,
  expiration_time timestamp,
  PRIMARY KEY (id)  
);
