-- +goose Up
ALTER TABLE premarket_info
ADD COLUMN is_hided BOOLEAN DEFAULT FALSE,
ADD COLUMN short_url_name VARCHAR(255);

-- +goose Down
ALTER TABLE premarket_info
DROP COLUMN is_hided,
DROP COLUMN short_url_name;
