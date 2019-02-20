DROP FUNCTION create_collpvc;
DROP FUNCTION create_object_id_fn_ext_mod_cb_collpvc;

DELETE FROM properties
WHERE id = 20 OR display = 'Collection';

ALTER TABLE properties
DROP CONSTRAINT "Name is unique";
