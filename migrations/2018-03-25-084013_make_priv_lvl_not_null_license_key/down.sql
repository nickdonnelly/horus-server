-- This file should undo anything in `up.sql`
ALTER TABLE horus_license_keys ALTER COLUMN privilege_level DROP NOT NULL;
ALTER TABLE horus_license_keys ALTER COLUMN privilege_level SET DEFAULT 1; 
