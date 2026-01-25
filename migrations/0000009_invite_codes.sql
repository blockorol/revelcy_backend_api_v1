-- +goose Up

CREATE TABLE invite_codes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    code TEXT NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    note TEXT
);

CREATE INDEX idx_invite_codes_owner_user_id ON invite_codes(owner_user_id);

CREATE TABLE user_invites (
    user_id UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    invite_code_id UUID NOT NULL REFERENCES invite_codes(id),
    applied_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_user_invites_invite_code_id ON user_invites(invite_code_id);

-- +goose Down
DROP TABLE IF EXISTS user_invites;
DROP TABLE IF EXISTS invite_codes;
