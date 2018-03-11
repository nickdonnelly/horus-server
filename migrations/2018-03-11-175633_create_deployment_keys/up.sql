-- Your SQL goes here
CREATE TABLE deployment_keys(
  key character varying(100) NOT NULL PRIMARY KEY,
  deployments integer NOT NULL DEFAULT 0,
  CONSTRAINT min_key_len check (length(key) >= 50)
);
