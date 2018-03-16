-- This file should undo anything in `up.sql`
ALTER TABLE horus_jobs DROP COLUMN IF EXISTS priority;
ALTER TABLE horus_jobs ALTER COLUMN job_data TYPE text;
ALTER TABLE horus_jobs ALTER COLUMN job_data SET NOT NULL;
