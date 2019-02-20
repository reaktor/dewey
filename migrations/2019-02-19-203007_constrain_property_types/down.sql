ALTER TABLE choice_values
DROP CONSTRAINT "Property must be choice type";

ALTER TABLE text_values
DROP CONSTRAINT "Property must be text type";

ALTER TABLE relation_values
DROP CONSTRAINT "Property must be relation type";

ALTER TABLE property_value_choices
DROP CONSTRAINT "Property must be choice type";

DROP FUNCTION property_type_is;
