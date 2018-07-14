-- This file should undo anything in `up.sql`
ALTER TABLE horus_images DROP COLUMN IF EXISTS password;
ALTER TABLE horus_files DROP COLUMN IF EXISTS password;
ALTER TABLE horus_videos DROP COLUMN IF EXISTS password;
