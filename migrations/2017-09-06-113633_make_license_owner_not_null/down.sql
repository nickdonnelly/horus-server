-- This file should undo anything in `up.sql`
ALTER TABLE horus_licenses ALTER COLUMN owner DROP NOT NULL;
