-- Your SQL goes here
ALTER TABLE horus_licenses ADD COLUMN resource_count integer NOT NULL default 0;

-- Images 
UPDATE horus_licenses SET resource_count = resource_count + (SELECT COUNT(*) FROM horus_images WHERE owner = horus_licenses.owner);

-- Videos
UPDATE horus_licenses SET resource_count = resource_count + (SELECT COUNT(*) FROM horus_videos WHERE owner = horus_licenses.owner);

-- Pastes
UPDATE horus_licenses SET resource_count = resource_count + (SELECT COUNT(*) FROM horus_pastes WHERE owner = horus_licenses.owner);

-- Files
UPDATE horus_licenses SET resource_count = resource_count + (SELECT COUNT(*) FROM horus_files WHERE owner = horus_licenses.owner);
