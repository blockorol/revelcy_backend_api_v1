-- +goose Up
-- +goose StatementBegin

CREATE TABLE IF NOT EXISTS vesting_info (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  premarket_id uuid NOT NULL REFERENCES premarket_info(id) ON DELETE CASCADE,
  vesting_address text NOT NULL UNIQUE,
  vesting_period bigint NOT NULL,
  init_unlock bigint NOT NULL,
  timestamp_start bigint NULL,
  timestamp_end bigint NULL,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_vesting_info_premarket_id 
  ON vesting_info(premarket_id);

CREATE INDEX IF NOT EXISTS idx_vesting_info_vesting_address 
  ON vesting_info(vesting_address);

CREATE TABLE IF NOT EXISTS vesting_holders (
  id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
  vesting_info_id uuid NOT NULL REFERENCES vesting_info(id) ON DELETE CASCADE,
  holder_id uuid NULL REFERENCES users(id),
  holder_wallet text NOT NULL,
  tokens_total bigint NOT NULL DEFAULT 0,
  tokens_claimed bigint NOT NULL DEFAULT 0,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE(vesting_info_id, holder_wallet)
);

CREATE INDEX IF NOT EXISTS idx_vesting_holders_vesting_info_id 
  ON vesting_holders(vesting_info_id);

CREATE INDEX IF NOT EXISTS idx_vesting_holders_holder_id 
  ON vesting_holders(holder_id);

CREATE INDEX IF NOT EXISTS idx_vesting_holders_holder_wallet 
  ON vesting_holders(holder_wallet);

-- Composite index for common queries
CREATE INDEX IF NOT EXISTS idx_vesting_holders_vesting_holder 
  ON vesting_holders(vesting_info_id, holder_wallet);

-- +goose StatementEnd

-- +goose Down
-- +goose StatementBegin

DROP INDEX IF EXISTS idx_vesting_holders_vesting_holder;
DROP INDEX IF EXISTS idx_vesting_holders_holder_wallet;
DROP INDEX IF EXISTS idx_vesting_holders_holder_id;
DROP INDEX IF EXISTS idx_vesting_holders_vesting_info_id;
DROP TABLE IF EXISTS vesting_holders;

DROP INDEX IF EXISTS idx_vesting_info_vesting_address;
DROP INDEX IF EXISTS idx_vesting_info_premarket_id;
DROP TABLE IF EXISTS vesting_info;

-- +goose StatementEnd