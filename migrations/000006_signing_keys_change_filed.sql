ALTER TABLE signing_keys
  DROP COLUMN IF EXISTS id_premarket,
  ADD COLUMN premarket_pubkey TEXT NOT NULL;
