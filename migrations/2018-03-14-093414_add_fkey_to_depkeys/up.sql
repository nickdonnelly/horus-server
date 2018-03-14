-- Your SQL goes here
ALTER TABLE deployment_keys 
ADD COLUMN license_key char(32) NOT NULL,
ADD CONSTRAINT f_lkey FOREIGN KEY (license_key) REFERENCES horus_license_keys (key) ON DELETE CASCADE;
