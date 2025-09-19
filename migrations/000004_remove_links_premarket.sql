ALTER TABLE premarket_info
  DROP COLUMN IF EXISTS telegram_team_chat,
  DROP COLUMN IF EXISTS instagram;

ALTER TABLE premarket_info
  RENAME COLUMN telegram_main_chat TO telegram;
