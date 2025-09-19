CREATE EXTENSION IF NOT EXISTS "pgcrypto";

CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username TEXT,
    avatar_url TEXT
);

CREATE TABLE IF NOT EXISTS wallets (
    user_id UUID REFERENCES users(id),
    wallet_address TEXT UNIQUE NOT NULL
);