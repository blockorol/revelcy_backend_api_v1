-- +goose Up

-- community_info(id) -> premarket_info(id)
ALTER TABLE community_info
    DROP CONSTRAINT IF EXISTS community_info_id_fkey;

ALTER TABLE community_info
    ADD CONSTRAINT community_info_id_fkey
    FOREIGN KEY (id) REFERENCES premarket_info(id)
    ON DELETE CASCADE;

-- community_links(community_info_id) -> community_info(id)
ALTER TABLE community_links
    DROP CONSTRAINT IF EXISTS community_links_community_info_id_fkey;

ALTER TABLE community_links
    ADD CONSTRAINT community_links_community_info_id_fkey
    FOREIGN KEY (community_info_id) REFERENCES community_info(id)
    ON DELETE CASCADE;

-- +goose Down

ALTER TABLE community_links
    DROP CONSTRAINT IF EXISTS community_links_community_info_id_fkey;

ALTER TABLE community_links
    ADD CONSTRAINT community_links_community_info_id_fkey
    FOREIGN KEY (community_info_id) REFERENCES community_info(id);

ALTER TABLE community_info
    DROP CONSTRAINT IF EXISTS community_info_id_fkey;

ALTER TABLE community_info
    ADD CONSTRAINT community_info_id_fkey
    FOREIGN KEY (id) REFERENCES premarket_info(id);
