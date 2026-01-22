-- +goose Up
ALTER TABLE premarket_info
ADD COLUMN is_whitelist_enabled BOOLEAN DEFAULT FALSE;

-- +goose Down
ALTER TABLE premarket_info
DROP COLUMN is_whitelist_enabled;
