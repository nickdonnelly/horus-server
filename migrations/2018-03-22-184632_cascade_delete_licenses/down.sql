-- This file should undo anything in `up.sql`
ALTER TABLE horus_licenses DROP CONSTRAINT horus_licenses_key_fkey;
ALTER TABLE horus_licenses DROP CONSTRAINT horus_owner_fkey;
ALTER TABLE session_tokens DROP CONSTRAINT session_tokens_uid_fkey;
ALTER TABLE horus_images DROP CONSTRAINT horus_images_owner_fkey;
ALTER TABLE horus_videos DROP CONSTRAINT horus_videos_owner_fkey;
ALTER TABLE horus_files DROP CONSTRAINT horus_files_owner_fkey;
ALTER TABLE horus_pastes DROP CONSTRAINT horus_pastes_owner_fkey;


-- readd them without cascade
ALTER TABLE horus_licenses ADD CONSTRAINT horus_licenses_key_fkey FOREIGN KEY (key) REFERENCES horus_license_keys(key) ON DELETE NO ACTION;
ALTER TABLE horus_licenses ADD CONSTRAINT horus_licenses_owner_fkey FOREIGN KEY (owner) REFERENCES horus_users(id) ON DELETE NO ACTION;
ALTER TABLE session_tokens ADD CONSTRAINT session_tokens_uid_fkey FOREIGN KEY (uid) REFERENCES horus_users(id) ON DELETE NO ACTION;
ALTER TABLE horus_videos ADD CONSTRAINT horus_videos_owner_fkey FOREIGN KEY (owner) REFERENCES horus_users(id) ON DELETE NO ACTION;
ALTER TABLE horus_images ADD CONSTRAINT horus_images_owner_fkey FOREIGN KEY (owner) REFERENCES horus_users(id) ON DELETE NO ACTION;
ALTER TABLE horus_files ADD CONSTRAINT horus_pastes_owner_fkey FOREIGN KEY (owner) REFERENCES horus_users(id) ON DELETE NO ACTION;
ALTER TABLE horus_pastes ADD CONSTRAINT horus_files FOREIGN KEY (owner) REFERENCES horus_users(id) ON DELETE NO ACTION;
