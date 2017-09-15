-- Your SQL goes here
CREATE TABLE auth_tokens(
  uid INTEGER NOT NULL REFERENCES horus_users(id),
  token char(128) NOT NULL,
  expires timestamp default now() + interval '10 minutes',
  PRIMARY KEY (uid) -- This ensures at maximum 1 auth token for a given user exists at any time.
);

CREATE TABLE session_tokens(
  uid INTEGER NOT NULL REFERENCES horus_users(id),
  token char(128) NOT NULL,
  use_count integer default 0,
  expires timestamp default now() + interval '72 hours',
  PRIMARY KEY (uid)
);
