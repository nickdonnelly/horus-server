-- This file should undo anything in `up.sql`
ALTER TABLE horus_jobs DROP CONSTRAINT horus_jobs_owner_fkey;

ALTER TABLE horus_jobs ADD CONSTRAINT horus_jobs_owner_fkey FOREIGN KEY (owner) REFERENCES horus_users (id);
