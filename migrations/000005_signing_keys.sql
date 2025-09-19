CREATE TABLE signing_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    id_premarket UUID REFERENCES premarket_info(id) ON DELETE CASCADE,
    pub_key TEXT NOT NULL,
    priv_key TEXT NOT NULL,
    type TEXT NOT NULL DEFAULT 'mint_key',
    UNIQUE (pub_key)
);
