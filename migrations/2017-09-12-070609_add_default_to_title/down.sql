-- This file should undo anything in `up.sql`
ALTER TABLE ONLY horus_images ALTER COLUMN title DROP DEFAULT;
ALTER TABLE ONLY horus_videos ALTER COLUMN title DROP DEFAULT;
ALTER TABLE ONLY horus_pastes ALTER COLUMN title DROP DEFAULT;
