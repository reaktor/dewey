INSERT INTO users(id, full_name, display_name)
VALUES
  (2, 'Unknown', 'Account Unknown');

CREATE TABLE objects (
  id TEXT PRIMARY KEY,
  created_by BIGINT NOT NULL DEFAULT 2 REFERENCES users(id) ON DELETE SET DEFAULT,
  created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TYPE property_type AS ENUM (
  'choice', 'text', 'relation', 'timestamptz'
);

CREATE FUNCTION properties_ord() RETURNS REAL AS $$
BEGIN RETURN 1.0::REAL; END;
$$ LANGUAGE plpgsql;

CREATE TABLE properties (
  id BIGINT PRIMARY KEY DEFAULT id_generator(),
  created_by BIGINT NOT NULL DEFAULT 2 REFERENCES users(id) ON DELETE SET DEFAULT,
  created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
  ord REAL NOT NULL DEFAULT properties_ord(),
  display TEXT NOT NULL CONSTRAINT "property name not empty" CHECK (display <> ''),
  property_type property_type NOT NULL
);

CREATE OR REPLACE FUNCTION properties_ord() RETURNS REAL AS $$
SELECT COUNT(*)::REAL FROM properties;
$$ LANGUAGE SQL;

-- These are the default properties which can be automatically
-- assigned by the system during upload.
INSERT INTO properties(id, created_by, display, property_type)
VALUES
  (1, 0, 'Filename', 'text'),
  (2, 0, 'Hash', 'text'),
  (3, 0, 'Last Modified', 'timestamptz'),
  (10, 0, 'Tags', 'choice');


CREATE TABLE property_value_choices(
  id BIGINT PRIMARY KEY DEFAULT id_generator(),
  property_id BIGINT NOT NULL REFERENCES properties(id) ON DELETE CASCADE,
  display TEXT NOT NULL CONSTRAINT "choice name not empty" CHECK (display <> ''),
  created_by BIGINT NOT NULL DEFAULT 2 REFERENCES users(id) ON DELETE SET DEFAULT,
  created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE choice_values(
  "object_id" TEXT NOT NULL REFERENCES objects(id) ON DELETE CASCADE,
  property_id BIGINT NOT NULL REFERENCES properties(id) ON DELETE CASCADE,
  value_id BIGINT REFERENCES property_value_choices(id) ON DELETE CASCADE,
  created_by BIGINT NOT NULL DEFAULT 2 REFERENCES users(id) ON DELETE SET DEFAULT,
  created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY ("object_id", property_id, value_id)
);

CREATE TABLE text_values(
  "object_id" TEXT NOT NULL REFERENCES objects(id) ON DELETE CASCADE,
  property_id BIGINT NOT NULL REFERENCES properties(id) ON DELETE CASCADE,
  "value" TEXT NOT NULL DEFAULT '',
  created_by BIGINT NOT NULL DEFAULT 2 REFERENCES users(id) ON DELETE SET DEFAULT,
  created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY ("object_id", property_id)
);

CREATE TABLE relation_values(
  "object_id" TEXT NOT NULL REFERENCES objects(id) ON DELETE CASCADE,
  property_id BIGINT NOT NULL REFERENCES properties(id) ON DELETE CASCADE,
  target_id TEXT NOT NULL REFERENCES objects(id) ON DELETE CASCADE,
  created_by BIGINT NOT NULL DEFAULT 2 REFERENCES users(id) ON DELETE SET DEFAULT,
  created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY ("object_id", property_id, target_id)
);

-- SEED CHOICES
INSERT INTO property_value_choices(property_id, display, created_by)
VALUES
  (10, 'Illustration', 0),
  (10, 'Branding', 0),
  (10, 'Proposal', 0),
  (10, 'Presentation', 0),
  (10, 'Wireframe', 0),
  (10, 'Web UI', 0),
  (10, 'Mobile UI', 0),
  (10, 'Codebase', 0);
