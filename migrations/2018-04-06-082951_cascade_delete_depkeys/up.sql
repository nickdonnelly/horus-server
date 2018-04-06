-- Your SQL goes here
ALTER TABLE deployment_keys ADD CONSTRAINT unique_hash UNIQUE (key);
