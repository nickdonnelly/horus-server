-- This file should undo anything in `up.sql`
ALTER TABLE horus_jobs ALTER COLUMN time_queued DROP NOT NULL;
