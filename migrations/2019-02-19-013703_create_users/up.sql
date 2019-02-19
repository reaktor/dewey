CREATE SEQUENCE global_id_sequence;

CREATE OR REPLACE FUNCTION id_generator(OUT result bigint) AS $$
DECLARE
    our_epoch bigint := 1314220021721;
    seq_id bigint;
    now_millis bigint;
    -- the id of this DB shard, must be set for each
    -- schema shard you have - you could pass this as a parameter too
    shard_id int := 1;
BEGIN
    SELECT nextval('global_id_sequence') % 1024 INTO seq_id;

    SELECT FLOOR(EXTRACT(EPOCH FROM clock_timestamp()) * 1000) INTO now_millis;
    result := (now_millis - our_epoch) << 23;
    result := result | (shard_id << 10);
    result := result | (seq_id);
END;
$$ LANGUAGE PLPGSQL;

CREATE TABLE users (
  id BIGINT PRIMARY KEY DEFAULT id_generator(),
  google_resource_id TEXT,
  full_name TEXT NOT NULL,
  display_name TEXT NOT NULL
);

INSERT INTO users(id, full_name, display_name)
VALUES
  (0, 'App', 'Application Defaults'),
  (1, 'Admin', 'Administrator');

INSERT INTO users(full_name, display_name)
VALUES
  ('Cole', 'Cole Lawrence');
