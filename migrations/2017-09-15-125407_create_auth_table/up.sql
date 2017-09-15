-- Your SQL goes here
CREATE TABLE auth_tokens(
  id INTEGER NOT NULL REFERENCES horus_users(id),
  token char(128) NOT NULL,
  use_count integer default 0, --Resets every 25 requests?
  PRIMARY KEY (id) -- This ensures at maximum 1 auth token for a given user exists at any time.
);
