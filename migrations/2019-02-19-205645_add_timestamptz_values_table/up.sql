
CREATE TABLE timestamptz_values(
  "object_id" TEXT NOT NULL REFERENCES objects(id) ON DELETE CASCADE,
  property_id BIGINT NOT NULL REFERENCES properties(id) ON DELETE CASCADE,
  "value" TIMESTAMPTZ,
  created_by BIGINT NOT NULL DEFAULT 2 REFERENCES users(id) ON DELETE SET DEFAULT,
  created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY ("object_id", property_id),
  CONSTRAINT "Property must be timestamptz type"
    CHECK (property_type_is(property_id, 'timestamptz'))
);
