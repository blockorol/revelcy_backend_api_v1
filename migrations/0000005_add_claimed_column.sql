-- +goose Up
ALTER TABLE premarket_holders
ADD COLUMN claimed BOOLEAN NOT NULL DEFAULT false;

CREATE INDEX idx_holders_claimed ON premarket_holders(claimed) WHERE claimed = false;

-- +goose Down
DROP INDEX IF EXISTS idx_holders_claimed;
ALTER TABLE premarket_holders
DROP COLUMN IF EXISTS claimed;

