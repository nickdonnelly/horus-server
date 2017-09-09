-- This file should undo anything in `up.sql`
ALTER TABLE ONLY horus_images ALTER COLUMN date_added DROP DEFAULT;
ALTER TABLE ONLY horus_videos ALTER COLUMN date_added DROP DEFAULT;
ALTER TABLE ONLY horus_pastes ALTER COLUMN date_added DROP DEFAULT;
