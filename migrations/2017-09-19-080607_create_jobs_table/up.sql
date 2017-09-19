-- Your SQL goes here
CREATE TABLE horus_jobs (
  id SERIAL NOT NULL,
  owner integer NOT NULL REFERENCES horus_users(id),
  job_status integer NOT NULL DEFAULT 0, -- default waiting/"queued"
  job_name varchar(32) NOT NULL,
  job_data text NOT NULL, 
  time_queued timestamp DEFAULT now(),
  PRIMARY KEY(id)
);
