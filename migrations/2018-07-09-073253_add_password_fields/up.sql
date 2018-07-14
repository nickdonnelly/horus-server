-- Your SQL goes here
ALTER TABLE horus_images ADD COLUMN password varchar;
ALTER TABLE horus_files ADD COLUMN password varchar;
ALTER TABLE horus_videos ADD COLUMN password varchar;
