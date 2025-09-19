CREATE TABLE IF NOT EXISTS premarket_holders (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    premarket_info_id UUID NOT NULL REFERENCES premarket_info(id) ON DELETE CASCADE,
    holder_id UUID,
    holder_wallet TEXT NOT NULL,
    amount_lamport BIGINT NOT NULL,
    join_timestamp BIGINT NOT NULL,
    out_timestamp BIGINT
);

CREATE INDEX IF NOT EXISTS idx_holders_by_premarket ON premarket_holders(premarket_info_id);
CREATE INDEX IF NOT EXISTS idx_holder_wallet ON premarket_holders(holder_wallet);
