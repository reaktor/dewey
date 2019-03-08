CREATE TABLE forms (
  id BIGINT PRIMARY KEY DEFAULT id_generator(),
  handle TEXT UNIQUE,
  title TEXT,
  listed BOOLEAN NOT NULL,
  created_by BIGINT NOT NULL DEFAULT 2 REFERENCES users(id) ON DELETE SET DEFAULT,
  created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE FUNCTION form_properties_ord() RETURNS REAL AS $$
BEGIN RETURN 1.0::REAL; END;
$$ LANGUAGE plpgsql;

CREATE TABLE form_properties (
  form_id BIGINT NOT NULL REFERENCES forms(id) ON DELETE CASCADE,
  property_id BIGINT NOT NULL REFERENCES properties(id) ON DELETE CASCADE,
  ord REAL NOT NULL DEFAULT form_properties_ord(),
  description TEXT,
  PRIMARY KEY (form_id, property_id)
);

CREATE TABLE form_default_choice_properties (
  form_id BIGINT NOT NULL REFERENCES forms(id) ON DELETE CASCADE,
  property_id BIGINT NOT NULL REFERENCES properties(id) ON DELETE CASCADE,
  property_value_choice_id BIGINT NOT NULL REFERENCES property_value_choices(id) ON DELETE CASCADE,
  PRIMARY KEY (form_id, property_id, property_value_choice_id),
  CONSTRAINT "Property default must be choice type"
    CHECK (property_type_is(property_id, 'choice'))
);

CREATE FUNCTION form_has_property(form_id bigint, property_id bigint) RETURNS BOOLEAN AS $$
SELECT COUNT(*) = 1 FROM form_properties
WHERE form_id = form_id AND property_id = property_id;
$$ LANGUAGE SQL;

CREATE TABLE form_nested_forms (
  form_id BIGINT NOT NULL REFERENCES forms(id) ON DELETE CASCADE,
  property_id BIGINT NOT NULL REFERENCES properties(id) ON DELETE CASCADE,
  nested_form_id BIGINT NOT NULL REFERENCES forms(id) ON DELETE CASCADE,
  -- Notice that there can only be one nested form per property
  PRIMARY KEY (form_id, property_id),
  CONSTRAINT "Property for nested form must be relation type"
    CHECK (property_type_is(property_id, 'relation')),
  CONSTRAINT "Property must exist on parent form"
    CHECK (form_has_property(form_id, property_id))
);

-- Make sure that we actually put this ord in the back
CREATE OR REPLACE FUNCTION form_properties_ord() RETURNS real AS $$
SELECT (COALESCE(MAX(ord), 1) + 1)::REAL FROM form_properties;
$$ LANGUAGE SQL;


-- Seed values for first form

-- 1. Adam creates a new collection form
-- -- Create form: Tokyo Proposals
INSERT INTO forms
  (id, handle, title, listed, created_by)
VALUES
  (1901, 'test-jp-proposals', 'Tokyo Proposals 🇯🇵🏯', TRUE, 901);
-- -- Create collection: Tokyo Proposals (Same name)
INSERT INTO property_value_choices
  (id, property_id, display, created_by)
VALUES
  (2001, 20, 'Tokyo Proposals 🇯🇵🏯', 901);
-- -- Add collection choice property as a default property value
INSERT INTO form_default_choice_properties
  (form_id, property_id, property_value_choice_id)
VALUES
  (1901, 20, 2001);
-- 2. Adam wants to people to add any context necessary to this form as text
-- 2.1 Adam must create a property which does not yet exist, he'll call it "Context"/text
-- -- Create Context property
INSERT INTO properties
  (id, display, property_type, created_by)
VALUES
  (901961, 'Context', 'text', 901);
-- -- Add property to form
INSERT INTO form_properties
  (form_id, property_id, description)
VALUES
  (1901, 901961, 'What else should we know about this proposal and how it was used?');
-- 3. Adam wants people to also submit any source files used to create the proposal
-- 3.1 Adam must create a property which does not yet exist, he'll call it "Source files"/relation
-- -- Create "Source files" relation property
INSERT INTO properties
  (id, display, property_type, created_by)
VALUES
  (901981, 'Source files', 'relation', 901);
-- -- Add property to form
INSERT INTO form_properties
  (form_id, property_id, description)
VALUES
  (1901, 901981, 'Please share any source files used for this proposal ❤️');
