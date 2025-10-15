-- +goose Up
ALTER TABLE premarket_info
ADD COLUMN finish_deadline BIGINT NULL,
ADD COLUMN mint_address TEXT NOT NULL DEFAULT 'None';

-- remove DEFAULT,to change it to reqiuired field
ALTER TABLE premarket_info
ALTER COLUMN mint_address DROP DEFAULT;

-- +goose Down
ALTER TABLE premarket_info
DROP COLUMN IF EXISTS mint_address,
DROP COLUMN IF EXISTS finish_deadline;
