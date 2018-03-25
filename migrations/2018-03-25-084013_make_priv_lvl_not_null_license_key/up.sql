-- Your SQL goes here
ALTER TABLE horus_license_keys ALTER COLUMN privilege_level SET NOT NULL;
ALTER TABLE horus_license_keys ALTER COLUMN privilege_level SET DEFAULT 0;
