-- +goose Up
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username TEXT,
    avatar_url TEXT
);

CREATE TABLE wallets (
    user_id UUID REFERENCES users(id),
    wallet_address TEXT UNIQUE NOT NULL
);

CREATE TABLE premarket_info (
    id UUID PRIMARY KEY,
    creator_id UUID NOT NULL,
    creator_address TEXT NOT NULL,
    bc_address TEXT NOT NULL,
    data_uri TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    symbol TEXT NOT NULL,
    image_url TEXT,
    telegram TEXT,
    twitter TEXT,
    web_site TEXT,
    premarket_goal_pers DOUBLE PRECISION NOT NULL,
    premarket_goal_sol_lamp BIGINT NOT NULL,
    premarket_deadline BIGINT NOT NULL,
    premarket_created BIGINT NOT NULL,
    state TEXT NOT NULL
);

CREATE TABLE community_info (
    id UUID PRIMARY KEY REFERENCES premarket_info(id),
    description TEXT NOT NULL,
    token_banner_url TEXT
);

CREATE TABLE community_links (
    id UUID PRIMARY KEY,
    community_info_id UUID NOT NULL REFERENCES community_info(id),
    text TEXT NOT NULL,
    url TEXT NOT NULL,
    type TEXT NOT NULL
);

CREATE TABLE premarket_holders (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    premarket_info_id UUID NOT NULL REFERENCES premarket_info(id) ON DELETE CASCADE,
    holder_id UUID,
    holder_wallet TEXT NOT NULL,
    amount_lamport BIGINT NOT NULL,
    join_timestamp BIGINT NOT NULL,
    out_timestamp BIGINT
);

CREATE INDEX idx_holders_by_premarket ON premarket_holders(premarket_info_id);
CREATE INDEX idx_holder_wallet ON premarket_holders(holder_wallet);

CREATE TABLE signing_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    pub_key TEXT NOT NULL,
    priv_key TEXT NOT NULL,
    type TEXT NOT NULL DEFAULT 'mint_key',
    premarket_pubkey TEXT NOT NULL,
    UNIQUE (pub_key)
);

-- +goose Down
DROP TABLE IF EXISTS signing_keys;
DROP TABLE IF EXISTS premarket_holders;
DROP TABLE IF EXISTS community_links;
DROP TABLE IF EXISTS community_info;
DROP TABLE IF EXISTS premarket_info;
DROP TABLE IF EXISTS wallets;
DROP TABLE IF EXISTS users;
DROP EXTENSION IF EXISTS "pgcrypto";
