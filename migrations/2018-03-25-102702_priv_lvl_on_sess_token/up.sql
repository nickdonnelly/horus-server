-- Your SQL goes here
ALTER TABLE session_tokens ADD COLUMN privilege_level integer DEFAULT 0 NOT NULL;
ALTER TABLE auth_tokens ADD COLUMN privilege_level integer DEFAULT 0 NOT NULL;
