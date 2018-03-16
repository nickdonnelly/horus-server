-- Your SQL goes here
ALTER TABLE horus_jobs ADD COLUMN priority integer NOT NULL DEFAULT 0;
ALTER TABLE horus_jobs ALTER COLUMN job_data TYPE bytea USING (trim(job_data)::bytea);
ALTER TABLE horus_jobs ALTER COLUMN job_data DROP NOT NULL;
