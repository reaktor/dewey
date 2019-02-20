DROP TABLE relation_values;
DROP TABLE text_values;
DROP TABLE choice_values;
DROP TABLE property_value_choices;
DROP TABLE properties;

DROP FUNCTION properties_ord;
DROP TYPE property_type;
DROP TABLE objects;

DELETE FROM users
WHERE id = 2 OR id = 3;
