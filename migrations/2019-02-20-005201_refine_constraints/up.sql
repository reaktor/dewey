ALTER TABLE property_value_choices
ADD CONSTRAINT "Choice name must be unique" UNIQUE(property_id, display);
