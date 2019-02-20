ALTER TABLE objects
ADD COLUMN extension TEXT NOT NULL DEFAULT '';

-- Basic new object with attributes function
CREATE FUNCTION create_object_id_fn_ext_mod_cb
  (object_id text, "filename" text, extension text, modified timestamptz, created_by bigint)
RETURNS void AS $$

-- Create object
INSERT INTO objects
  (id, extension, created_by)
VALUES
  (object_id, extension, created_by);

-- Assign filename property
INSERT INTO text_values
  (object_id, property_id, value, created_by)
VALUES
  (object_id, 1, "filename", created_by);

-- Assign last modified property
INSERT INTO timestamptz_values
  (object_id, property_id, value, created_by)
VALUES
  (object_id, 3, modified, created_by);

$$ LANGUAGE SQL;

-- Make sure that we actually put this ord in the back
CREATE OR REPLACE FUNCTION properties_ord() RETURNS real AS $$
SELECT (MAX(ord) + 1)::REAL FROM properties;
$$ LANGUAGE SQL;
