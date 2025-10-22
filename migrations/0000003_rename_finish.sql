-- +goose Up
ALTER TABLE premarket_info
RENAME COLUMN finish_deadline TO premarket_finished;

-- +goose Down
ALTER TABLE premarket_info
RENAME COLUMN premarket_finished TO finish_deadline;
