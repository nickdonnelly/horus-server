-- This file should undo anything in `up.sql`
ALTER TABLE horus_versions DROP CONSTRAINT horus_versions_pkey;
ALTER TABLE horus_versions ADD PRIMARY KEY (id);
