-- +goose Up
-- +goose StatementBegin

CREATE TABLE IF NOT EXISTS vesting_info (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  premarket_id uuid NOT NULL UNIQUE REFERENCES premarket_info(id) ON DELETE CASCADE,
  vesting_address text NOT NULL UNIQUE,
  vesting_period bigint NOT NULL,
  init_unlock bigint NOT NULL,
  timestamp_start bigint DEFAULT NULL,
  timestamp_end bigint DEFAULT NULL,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

-- Add vesting-related fields to premarket_holders
ALTER TABLE premarket_holders 
  ADD COLUMN IF NOT EXISTS amount_token BIGINT NOT NULL DEFAULT 0,
  ADD COLUMN IF NOT EXISTS claimed_amount_token BIGINT NOT NULL DEFAULT 0,
  ADD COLUMN IF NOT EXISTS updated_at timestamptz NOT NULL DEFAULT now();

CREATE INDEX IF NOT EXISTS idx_premarket_holders_holder_id 
  ON premarket_holders(holder_id);

-- +goose StatementEnd

-- +goose Down
-- +goose StatementBegin

DROP INDEX IF EXISTS idx_premarket_holders_holder_id;


ALTER TABLE premarket_holders 
  DROP COLUMN IF EXISTS updated_at,
  DROP COLUMN IF EXISTS claimed_amount_token,
  DROP COLUMN IF EXISTS amount_token;

DROP TABLE IF EXISTS vesting_info;

-- +goose StatementEnd