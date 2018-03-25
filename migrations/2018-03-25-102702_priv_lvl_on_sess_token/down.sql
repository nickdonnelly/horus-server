-- This file should undo anything in `up.sql`
ALTER TABLE session_tokens DROP COLUMN privilege_level;
ALTER TABLE auth_tokens DROP COLUMN privilege_level;
