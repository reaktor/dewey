-- Intermediate tokens should be tracked through Redis
-- This is the more long term storage for refresh tokens and such.
CREATE TABLE user_tokens(
  "user_id" BIGINT PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
  google_resource_id TEXT NOT NULL UNIQUE,
  -- Use version to invalidate previous sessions 
  version INT NOT NULL DEFAULT 0,
  created_at TIMESTAMPTZ NOT NULL,
  access_token TEXT NOT NULL,
  refresh_token TEXT NOT NULL,
  token_expiration TIMESTAMPTZ NOT NULL,
  CONSTRAINT "Access token must not be empty" CHECK (access_token <> ''),
  CONSTRAINT "Refresh token must not be empty" CHECK (refresh_token <> '')
);

CREATE UNIQUE INDEX ON user_tokens (google_resource_id);
