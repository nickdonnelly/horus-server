-- This file should undo anything in `up.sql`
ALTER TABLE deployment_keys DROP CONSTRAINT unique_hash;
