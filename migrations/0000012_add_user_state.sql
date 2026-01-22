-- +goose Up
ALTER TABLE users
ADD COLUMN status TEXT NOT NULL DEFAULT 'REGISTERED';

-- +goose Down
ALTER TABLE users
DROP COLUMN status;
