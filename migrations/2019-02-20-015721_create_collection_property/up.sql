ALTER TABLE properties
ADD CONSTRAINT "Name is unique" UNIQUE(display);

INSERT INTO properties
  (id, created_by, display, property_type)
VALUES
  (20, 0, 'Collection', 'choice');


-- Basic new object with collections function
CREATE FUNCTION create_object_id_fn_ext_mod_cb_collpvc
  (object_id text, "filename" text, extension text, modified timestamptz, created_by bigint, collection_id bigint)
RETURNS void AS $$

-- create initial object and attributes
SELECT create_object_id_fn_ext_mod_cb(object_id, "filename", extension, modified, created_by);

-- Assign collection property
INSERT INTO choice_values
  (object_id, property_id, created_by, value_id)
VALUES
  (object_id, 20, created_by, collection_id);
$$ LANGUAGE SQL;


-- Basic new collections function
CREATE FUNCTION create_collpvc
 (collection_name text, created_by bigint)
RETURNS bigint AS $$

-- Assign collection property
INSERT INTO property_value_choices
  (id, property_id, display, created_by)
VALUES
  (DEFAULT, 20, collection_name, created_by)
RETURNING id;

$$ LANGUAGE SQL;
