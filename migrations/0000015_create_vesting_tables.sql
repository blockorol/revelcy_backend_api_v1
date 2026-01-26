-- +goose Up
-- +goose StatementBegin

CREATE TABLE IF NOT EXISTS vesting_info (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  premarket_id uuid NOT NULL REFERENCES premarket_info(id) ON DELETE CASCADE,
  vesting_address text NOT NULL UNIQUE,
  vesting_period bigint NOT NULL,
  init_unlock bigint NOT NULL,
  timestamp_start bigint DEFAULT NULL,
  timestamp_end bigint DEFAULT NULL,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_vesting_info_premarket_id 
  ON vesting_info(premarket_id);

CREATE INDEX IF NOT EXISTS idx_vesting_info_vesting_address 
  ON vesting_info(vesting_address);

-- Add vesting-related fields to premarket_holders
ALTER TABLE premarket_holders 
  ADD COLUMN IF NOT EXISTS amount_token BIGINT DEFAULT 0,
  ADD COLUMN IF NOT EXISTS claimed_amount_token BIGINT DEFAULT 0,
  ADD COLUMN IF NOT EXISTS updated_at timestamptz DEFAULT now();

-- Update holder_id to have foreign key constraint
ALTER TABLE premarket_holders 
  DROP CONSTRAINT IF EXISTS premarket_holders_holder_id_fkey;


-- Add index for holder_id
CREATE INDEX IF NOT EXISTS idx_premarket_holders_holder_id 
  ON premarket_holders(holder_id);

-- +goose StatementEnd

-- +goose Down
-- +goose StatementBegin

DROP INDEX IF EXISTS idx_premarket_holders_holder_id;

ALTER TABLE premarket_holders 
  DROP CONSTRAINT IF EXISTS premarket_holders_holder_id_fkey;

ALTER TABLE premarket_holders 
  DROP COLUMN IF EXISTS updated_at,
  DROP COLUMN IF EXISTS claimed_amount_token,
  DROP COLUMN IF EXISTS amount_token;

DROP INDEX IF EXISTS idx_vesting_info_vesting_address;
DROP INDEX IF EXISTS idx_vesting_info_premarket_id;
DROP TABLE IF EXISTS vesting_info;

-- +goose StatementEnd