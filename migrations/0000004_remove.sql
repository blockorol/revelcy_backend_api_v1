-- +goose Up
ALTER TABLE premarket_info
DROP COLUMN premarket_goal_pers;

-- +goose Down
ALTER TABLE premarket_info
ADD COLUMN premarket_goal_pers DOUBLE PRECISION ;
