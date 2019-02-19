INSERT INTO users(id, full_name, display_name)
VALUES
  (2, 'Removed', 'Account Removed'),
  (3, 'Unknown', 'Account Unknown');

CREATE TABLE objects (
  id TEXT PRIMARY KEY,
  created_by BIGINT NOT NULL REFERENCES users(id) DEFAULT 3,
  created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE properties (
  id BIGINT PRIMARY KEY DEFAULT id_generator(),
  created_by BIGINT NOT NULL REFERENCES users(id) DEFAULT 3,
  created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
  display TEXT NOT NULL,
  "type" TEXT NOT NULL
);

-- These are the default properties which can be automatically
-- assigned by the system during upload.
INSERT INTO properties(id, created_by, display, "type")
VALUES
  (1, 0, 'Filename', 'text'),
  (2, 0, 'Hash', 'text'),
  (3, 0, 'Last Modified', 'date'),
  (4, 0, 'File Size', 'usize'),
  (5, 0, 'Tags', 'multiselect');

CREATE TABLE "values" (
  "object_id" TEXT NOT NULL REFERENCES objects(id),
  property_id BIGINT NOT NULL REFERENCES properties(id),
  created_by BIGINT NOT NULL REFERENCES users(id) DEFAULT 3,
  created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
  "value" TEXT NOT NULL,
  PRIMARY KEY ("object_id", property_id)
);

CREATE TABLE "property_select_choices" (
  id SERIAL,
  property_id BIGINT NOT NULL REFERENCES properties(id),
  display TEXT NOT NULL,
  created_by BIGINT NOT NULL REFERENCES users(id) DEFAULT 3,
  created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (id, property_id)
);

INSERT INTO "property_select_choices"(property_id, display, created_by)
VALUES
  (5, 'Illustration', 0),
  (5, 'Branding', 0),
  (5, 'Proposal', 0),
  (5, 'Presentation', 0),
  (5, 'Wireframe', 0),
  (5, 'Web UI', 0),
  (5, 'Mobile UI', 0),
  (5, 'Codebase', 0);
