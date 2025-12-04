-- +goose Up
ALTER TABLE premarket_holders
ADD COLUMN claimed BOOLEAN NOT NULL DEFAULT false;

-- +goose Down
ALTER TABLE premarket_holders
DROP COLUMN IF EXISTS claimed;

