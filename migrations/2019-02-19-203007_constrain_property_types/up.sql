
-- Validate type of property
CREATE FUNCTION property_type_is(bigint, property_type) RETURNS BOOL AS $$
SELECT COUNT(*) = 0 FROM properties
WHERE $1 = id AND property_type != $2;
$$ LANGUAGE SQL;

-- Remove all violating values
DELETE FROM property_value_choices WHERE property_type_is(property_id, 'choice') != true;
DELETE FROM choice_values WHERE property_type_is(property_id, 'choice') != true;
DELETE FROM text_values WHERE property_type_is(property_id, 'text') != true;
DELETE FROM relation_values WHERE property_type_is(property_id, 'relation') != true;

-- Create property type constraints
ALTER TABLE relation_values
ADD CONSTRAINT "Property must be relation type"
    CHECK (property_type_is(property_id, 'relation'));

ALTER TABLE property_value_choices
ADD CONSTRAINT "Property must be choice type"
    CHECK (property_type_is(property_id, 'choice'));

ALTER TABLE choice_values
ADD CONSTRAINT "Property must be choice type"
    CHECK (property_type_is(property_id, 'choice'));

ALTER TABLE text_values
ADD CONSTRAINT "Property must be text type"
    CHECK (property_type_is(property_id, 'text'));
