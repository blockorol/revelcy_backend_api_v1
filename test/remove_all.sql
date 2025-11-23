BEGIN;

--  REMOVE ALL DATA!!! JUST TO LOCAL RUN!!!!
TRUNCATE TABLE
  premarket_holders,
  premarket_info,
  wallets,
  users
RESTART IDENTITY CASCADE;

COMMIT;
