-- +goose Up
CREATE TABLE whitelist (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    premarket_id REFERENCES premarket_info(id) ON DELETE CASCADE,
    user_id REFERENCES users(id) ON DELETE CASCADE
);
CREATE UNIQUE INDEX IF NOT EXISTS whitelist_premarket_user_uidx
  ON whitelist (premarket_id, user_id);

-- +goose Down
DROP TABLE IF EXISTS whitelist;