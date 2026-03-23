-- +goose Up
ALTER TABLE premarket_info
ADD COLUMN is_concept_visible BOOLEAN NOT NULL DEFAULT FALSE,
ADD COLUMN concept_created TIMESTAMPTZ NOT NULL DEFAULT NOW();

-- +goose Down
ALTER TABLE premarket_info
DROP COLUMN concept_created,
DROP COLUMN is_concept_visible;
