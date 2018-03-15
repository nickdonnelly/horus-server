-- Your SQL goes here
CREATE TABLE horus_versions(
  id SERIAL NOT NULL,
  deployed_with character varying(100) NOT NULL,
  aws_bucket_path character varying NOT NULL,
  version_string character varying(20) NOT NULL,
  platform character(5) NOT NULL, -- linux or win64
  deploy_timestamp  timestamp NOT NULL DEFAULT now(),
  is_public boolean NOT NULL DEFAULT false,

  PRIMARY KEY (id),
  CONSTRAINT fkey_deployed_with FOREIGN KEY (deployed_with) REFERENCES deployment_keys (key)
    ON DELETE NO ACTION ON UPDATE NO ACTION
);
