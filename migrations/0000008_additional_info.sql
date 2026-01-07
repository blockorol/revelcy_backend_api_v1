-- +goose Up
-- +goose StatementBegin

CREATE TABLE IF NOT EXISTS user_fingerprint_events (
  id uuid PRIMARY KEY,
  user_id uuid NULL REFERENCES users(id),
  event_type text NOT NULL,

  client_ts_ms bigint NULL,
  server_ts_ms bigint NOT NULL,

  premarket text NULL,

  -- IMPORTANT: stable identifier for device/browser install
  install_id text NOT NULL DEFAULT 'default',
  install_id_source text NOT NULL DEFAULT 'default',

  ip text NOT NULL DEFAULT 'default',
  user_agent text NOT NULL DEFAULT 'default',
  accept_language text NOT NULL DEFAULT 'default',

  sec_ch_ua text NOT NULL DEFAULT 'default',
  sec_ch_ua_platform text NOT NULL DEFAULT 'default',
  sec_ch_ua_mobile text NOT NULL DEFAULT 'default',

  client jsonb NOT NULL DEFAULT '{}'::jsonb
);

CREATE INDEX IF NOT EXISTS idx_user_fingerprint_events_user_id_ts
  ON user_fingerprint_events(user_id, server_ts_ms DESC);

CREATE INDEX IF NOT EXISTS idx_user_fingerprint_events_install_id_ts
  ON user_fingerprint_events(install_id, server_ts_ms DESC);

-- +goose StatementEnd

-- +goose Down
-- +goose StatementBegin

DROP INDEX IF EXISTS idx_user_fingerprint_events_install_id_user_id;
DROP INDEX IF EXISTS idx_user_fingerprint_events_install_id_ts;
DROP INDEX IF EXISTS idx_user_fingerprint_events_user_id_ts;
DROP TABLE IF EXISTS user_fingerprint_events;

-- +goose StatementEnd