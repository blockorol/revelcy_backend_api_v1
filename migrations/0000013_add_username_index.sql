-- +goose Up
CREATE EXTENSION IF NOT EXISTS pg_trgm;

CREATE INDEX IF NOT EXISTS users_username_trgm_idx
ON users
USING gin (username gin_trgm_ops);

-- +goose Down
DROP INDEX IF EXISTS users_username_trgm_idx;
DROP EXTENSION IF EXISTS pg_trgm;
