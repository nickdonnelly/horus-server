-- This file should undo anything in `up.sql`
ALTER TABLE horus_jobs ALTER COLUMN priority SET DEFAULT 0;
