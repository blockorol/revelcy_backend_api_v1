CREATE TABLE IF NOT EXISTS premarket_info (
    id UUID PRIMARY KEY,
    creator_id UUID NOT NULL,
    creator_address TEXT NOT NULL,
    bc_address TEXT NOT NULL,
    data_uri TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    symbol TEXT NOT NULL,
    image_url TEXT,

    telegram_main_chat TEXT,
    telegram_team_chat TEXT,
    twitter TEXT,
    instagram TEXT,
    web_site TEXT,

    premarket_goal_pers DOUBLE PRECISION NOT NULL,
    premarket_goal_sol_lamp BIGINT NOT NULL,
    premarket_deadline BIGINT NOT NULL,
    premarket_created BIGINT NOT NULL,

    state TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS community_info (
    id UUID PRIMARY KEY REFERENCES premarket_info(id),
    description TEXT NOT NULL,
    token_banner_url TEXT
);

CREATE TABLE IF NOT EXISTS community_links (
    id UUID PRIMARY KEY,
    community_info_id UUID NOT NULL REFERENCES community_info(id),
    text TEXT NOT NULL,
    url TEXT NOT NULL,
    type TEXT NOT NULL
);
