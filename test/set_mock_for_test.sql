BEGIN;

-- ================== CLEANUP для повторных запусков ==================
DROP FUNCTION IF EXISTS tmp_add_holder(uuid, uuid, text, numeric, integer, boolean);
DROP TABLE IF EXISTS tmp_creators;
DROP TABLE IF EXISTS tmp_time;
DROP TABLE IF EXISTS tmp_const;

-- gen_random_uuid
CREATE EXTENSION IF NOT EXISTS pgcrypto;

-- ================== 1) Константы: 3 валидных base58 + LAMPORTS ======
CREATE TEMP TABLE tmp_const (
  c1_addr text,
  c2_addr text,
  c3_addr text,
  lamports_per_sol bigint
) ON COMMIT PRESERVE ROWS;

INSERT INTO tmp_const VALUES
('JsYzZFqB6faWhABMN66MF5QkzVNDAyQ8oD7tq8Rdt5o',
 '2EZHn53ZAS41W79s26f25kToxtSszTZPPdbYg95rFGGu',
 '9tvcbHAs8gfAvTeY95YNmLWWBz3UAGsysjMR72HjviTZ',
 1000000000);

-- ================== 2) Тайминги (epoch) =============================
CREATE TEMP TABLE tmp_time (
  now_epoch bigint,
  past_deadline bigint,
  future_deadline bigint,
  created_epoch bigint
) ON COMMIT PRESERVE ROWS;

INSERT INTO tmp_time
SELECT EXTRACT(EPOCH FROM now())::bigint,
       EXTRACT(EPOCH FROM now() - interval '1 day')::bigint,
       EXTRACT(EPOCH FROM now() + interval '30 day')::bigint,
       EXTRACT(EPOCH FROM now() - interval '7 day')::bigint;

-- ================== 3) Создатели: users + wallets ====================
WITH creators AS (
  SELECT c1_addr, c2_addr, c3_addr FROM tmp_const
)
-- если их кошельков нет — создаём пользователей
INSERT INTO users (id, username, avatar_url)
SELECT gen_random_uuid(), u.username, NULL
FROM (
  VALUES
    ('creator_1', (SELECT c1_addr FROM creators)),
    ('creator_2', (SELECT c2_addr FROM creators)),
    ('creator_3', (SELECT c3_addr FROM creators))
) AS u(username, addr)
LEFT JOIN wallets w ON w.wallet_address = u.addr
WHERE w.wallet_address IS NULL;

-- если пользователей нет — создаём по имени
INSERT INTO users (id, username, avatar_url)
SELECT gen_random_uuid(), sub.username, NULL
FROM (VALUES ('creator_1'), ('creator_2'), ('creator_3')) AS sub(username)
LEFT JOIN users u ON u.username = sub.username
WHERE u.id IS NULL;

-- кошельки для создателей
WITH c AS (SELECT c1_addr, c2_addr, c3_addr FROM tmp_const),
map AS (
  SELECT 'creator_1'::text AS uname, (SELECT c1_addr FROM c) AS addr
  UNION ALL SELECT 'creator_2', (SELECT c2_addr FROM c)
  UNION ALL SELECT 'creator_3', (SELECT c3_addr FROM c)
)
INSERT INTO wallets (user_id, wallet_address)
SELECT u.id, m.addr
FROM map m
JOIN users u ON u.username = m.uname
LEFT JOIN wallets w ON w.wallet_address = m.addr
WHERE w.wallet_address IS NULL
ON CONFLICT (wallet_address) DO NOTHING;

-- ================== 4) +17 обычных пользователей =====================
WITH gen AS (SELECT generate_series(1,17) AS n)
INSERT INTO users (id, username, avatar_url)
SELECT
  gen_random_uuid(),
  'u_' || substr(md5(random()::text || clock_timestamp()::text || n::text),1,10),
  CASE WHEN random() < 0.5 THEN
    (ARRAY[
      'https://images.unsplash.com/photo-1502685104226-ee32379fefbe',
      'https://images.unsplash.com/photo-1494790108377-be9c29b29330',
      'https://images.unsplash.com/photo-1527980965255-d3b416303d12',
      'https://images.unsplash.com/photo-1544005313-94ddf0286df2',
      'https://images.unsplash.com/photo-1547425260-76bcadfb4f2c',
      'https://images.unsplash.com/photo-1527980965259-88be1ef62262'
    ])[ (1 + floor(random()*6))::int ]
  ELSE NULL END
FROM gen
ON CONFLICT DO NOTHING;

-- ================== 5) tmp_creators (ids + addresses) ================
CREATE TEMP TABLE tmp_creators (
  c1_id uuid,
  c2_id uuid,
  c3_id uuid,
  c1_addr text,
  c2_addr text,
  c3_addr text
) ON COMMIT PRESERVE ROWS;

INSERT INTO tmp_creators
SELECT
  (SELECT u.id FROM users u JOIN wallets w ON w.user_id=u.id WHERE w.wallet_address=(SELECT c1_addr FROM tmp_const) LIMIT 1),
  (SELECT u.id FROM users u JOIN wallets w ON w.user_id=u.id WHERE w.wallet_address=(SELECT c2_addr FROM tmp_const) LIMIT 1),
  (SELECT u.id FROM users u JOIN wallets w ON w.user_id=u.id WHERE w.wallet_address=(SELECT c3_addr FROM tmp_const) LIMIT 1),
  (SELECT c1_addr FROM tmp_const),
  (SELECT c2_addr FROM tmp_const),
  (SELECT c3_addr FROM tmp_const);

DO $$
BEGIN
  IF (SELECT c1_id FROM tmp_creators) IS NULL THEN RAISE EXCEPTION 'Creator 1 not found'; END IF;
  IF (SELECT c2_id FROM tmp_creators) IS NULL THEN RAISE EXCEPTION 'Creator 2 not found'; END IF;
  IF (SELECT c3_id FROM tmp_creators) IS NULL THEN RAISE EXCEPTION 'Creator 3 not found'; END IF;
END $$;

-- ================== 6) PREMARKET_INFO (mint_address всегда NOT NULL) ==
-- ВНИМАНИЕ: тут НЕТ CTE, вместо этого в каждом INSERT:
-- FROM tmp_time t CROSS JOIN tmp_creators tc CROSS JOIN tmp_const c

-- A1 canceled
INSERT INTO premarket_info (
  id, creator_id, creator_address, bc_address, data_uri,
  name, description, symbol, image_url, telegram, twitter, web_site,
  premarket_goal_pers, premarket_goal_sol_lamp, premarket_deadline, premarket_created,
  state, mint_address, premarket_finished
)
SELECT gen_random_uuid(), tc.c1_id, tc.c1_addr, tc.c1_addr, 'ipfs://fake-A1',
       'F-Overdue-17-not-reached', 'fake: A1', 'FA1',
       'https://images.unsplash.com/photo-1518779578993-ec3579fee39f', NULL, NULL, NULL,
       17, 50 * c.lamports_per_sol, t.past_deadline, t.created_epoch,
       'canceled', tc.c1_addr, NULL
FROM tmp_time t
CROSS JOIN tmp_creators tc
CROSS JOIN tmp_const c;

-- A2 finished (mint = c2)
INSERT INTO premarket_info (
  id, creator_id, creator_address, bc_address, data_uri,
  name, description, symbol, image_url, telegram, twitter, web_site,
  premarket_goal_pers, premarket_goal_sol_lamp, premarket_deadline, premarket_created,
  state, mint_address, premarket_finished
)
SELECT gen_random_uuid(), tc.c1_id, tc.c1_addr, tc.c1_addr, 'ipfs://fake-A2',
       'F-Overdue-26-reached', 'fake: A2', 'FA2',
       'https://images.unsplash.com/photo-1503602642458-232111445657', NULL, NULL, NULL,
       26, 10 * c.lamports_per_sol, t.past_deadline, t.created_epoch,
       'finished', tc.c2_addr, EXTRACT(EPOCH FROM now() - interval '2 day')::bigint
FROM tmp_time t
CROSS JOIN tmp_creators tc
CROSS JOIN tmp_const c;

-- A3 canceled + finished set
INSERT INTO premarket_info (
  id, creator_id, creator_address, bc_address, data_uri,
  name, description, symbol, image_url, telegram, twitter, web_site,
  premarket_goal_pers, premarket_goal_sol_lamp, premarket_deadline, premarket_created,
  state, mint_address, premarket_finished
)
SELECT gen_random_uuid(), tc.c2_id, tc.c2_addr, tc.c2_addr, 'ipfs://fake-A3',
       'F-Overdue-11-expired', 'fake: A3', 'FA3',
       'https://images.unsplash.com/photo-1498050108023-c5249f4df085', NULL, NULL, NULL,
       11, 15 * c.lamports_per_sol, t.past_deadline, t.created_epoch,
       'canceled', tc.c2_addr, EXTRACT(EPOCH FROM now() - interval '1 day')::bigint
FROM tmp_time t
CROSS JOIN tmp_creators tc
CROSS JOIN tmp_const c;

-- A4 premarket
INSERT INTO premarket_info (
  id, creator_id, creator_address, bc_address, data_uri,
  name, description, symbol, image_url, telegram, twitter, web_site,
  premarket_goal_pers, premarket_goal_sol_lamp, premarket_deadline, premarket_created,
  state, mint_address, premarket_finished
)
SELECT gen_random_uuid(), tc.c3_id, tc.c3_addr, tc.c3_addr, 'ipfs://fake-A4',
       'F-Overdue-28-open', 'fake: A4', 'FA4',
       'https://images.unsplash.com/photo-1492724441997-5dc865305da7', NULL, NULL, NULL,
       28, 40 * c.lamports_per_sol, t.past_deadline, t.created_epoch,
       'premarket', tc.c3_addr, NULL
FROM tmp_time t
CROSS JOIN tmp_creators tc
CROSS JOIN tmp_const c;

-- B1 premarket
INSERT INTO premarket_info (
  id, creator_id, creator_address, bc_address, data_uri,
  name, description, symbol, image_url, telegram, twitter, web_site,
  premarket_goal_pers, premarket_goal_sol_lamp, premarket_deadline, premarket_created,
  state, mint_address, premarket_finished
)
SELECT gen_random_uuid(), tc.c1_id, tc.c1_addr, tc.c1_addr, 'ipfs://fake-B1',
       'F-Future-1-not-reached', 'fake: B1', 'FB1',
       'https://images.unsplash.com/photo-1472214103451-9374bd1c798e', NULL, NULL, NULL,
       1, 20 * c.lamports_per_sol, t.future_deadline, t.created_epoch,
       'premarket', tc.c1_addr, NULL
FROM tmp_time t
CROSS JOIN tmp_creators tc
CROSS JOIN tmp_const c;

-- B2 finished (mint = c2)
INSERT INTO premarket_info (
  id, creator_id, creator_address, bc_address, data_uri,
  name, description, symbol, image_url, telegram, twitter, web_site,
  premarket_goal_pers, premarket_goal_sol_lamp, premarket_deadline, premarket_created,
  state, mint_address, premarket_finished
)
SELECT gen_random_uuid(), tc.c2_id, tc.c2_addr, tc.c2_addr, 'ipfs://fake-B2',
       'F-Future-1-reached', 'fake: B2', 'FB2',
       'https://images.unsplash.com/photo-1454165804606-c3d57bc86b40', NULL, NULL, NULL,
       1, 3 * c.lamports_per_sol, t.future_deadline, t.created_epoch,
       'finished', tc.c2_addr, EXTRACT(EPOCH FROM now())::bigint
FROM tmp_time t
CROSS JOIN tmp_creators tc
CROSS JOIN tmp_const c;

-- B3 premarket
INSERT INTO premarket_info (
  id, creator_id, creator_address, bc_address, data_uri,
  name, description, symbol, image_url, telegram, twitter, web_site,
  premarket_goal_pers, premarket_goal_sol_lamp, premarket_deadline, premarket_created,
  state, mint_address, premarket_finished
)
SELECT gen_random_uuid(), tc.c3_id, tc.c3_addr, tc.c3_addr, 'ipfs://fake-B3',
       'F-Future-19-not-reached', 'fake: B3', 'FB3',
       'https://images.unsplash.com/photo-1529336953121-ad3a8e6fbb7d', NULL, NULL, NULL,
       19, 25 * c.lamports_per_sol, t.future_deadline, t.created_epoch,
       'premarket', tc.c3_addr, NULL
FROM tmp_time t
CROSS JOIN tmp_creators tc
CROSS JOIN tmp_const c;

-- B4 finished (mint = c1)
INSERT INTO premarket_info (
  id, creator_id, creator_address, bc_address, data_uri,
  name, description, symbol, image_url, telegram, twitter, web_site,
  premarket_goal_pers, premarket_goal_sol_lamp, premarket_deadline, premarket_created,
  state, mint_address, premarket_finished
)
SELECT gen_random_uuid(), tc.c1_id, tc.c1_addr, tc.c1_addr, 'ipfs://fake-B4',
       'F-Future-27-reached', 'fake: B4', 'FB4',
       'https://images.unsplash.com/photo-1522071820081-009f0129c71c', NULL, NULL, NULL,
       27, 12 * c.lamports_per_sol, t.future_deadline, t.created_epoch,
       'finished', tc.c1_addr, EXTRACT(EPOCH FROM now())::bigint
FROM tmp_time t
CROSS JOIN tmp_creators tc
CROSS JOIN tmp_const c;

-- C1 finished (mint = c2)
INSERT INTO premarket_info (
  id, creator_id, creator_address, bc_address, data_uri,
  name, description, symbol, image_url, telegram, twitter, web_site,
  premarket_goal_pers, premarket_goal_sol_lamp, premarket_deadline, premarket_created,
  state, mint_address, premarket_finished
)
SELECT gen_random_uuid(), tc.c2_id, tc.c2_addr, tc.c2_addr, 'ipfs://fake-C1',
       'F-Launched-25-reached', 'fake: C1', 'FC1',
       'https://images.unsplash.com/photo-1517245386807-bb43f82c33c4', NULL, NULL, NULL,
       25, 18 * c.lamports_per_sol, t.past_deadline, t.created_epoch,
       'finished', tc.c2_addr, EXTRACT(EPOCH FROM now() - interval '3 day')::bigint
FROM tmp_time t
CROSS JOIN tmp_creators tc
CROSS JOIN tmp_const c;

-- D1 about + image (premarket)
INSERT INTO premarket_info (
  id, creator_id, creator_address, bc_address, data_uri,
  name, description, symbol, image_url, telegram, twitter, web_site,
  premarket_goal_pers, premarket_goal_sol_lamp, premarket_deadline, premarket_created,
  state, mint_address, premarket_finished
)
SELECT gen_random_uuid(), tc.c1_id, tc.c1_addr, tc.c1_addr, 'ipfs://fake-D1',
       'F-About-With-Image', 'fake: community about here', 'FD1',
       'https://images.unsplash.com/photo-1520975916090-3105956dac38', NULL, NULL, 'https://example.com',
       5, 5 * c.lamports_per_sol, t.future_deadline, t.created_epoch,
       'premarket', tc.c1_addr, NULL
FROM tmp_time t
CROSS JOIN tmp_creators tc
CROSS JOIN tmp_const c;

-- D2 about, no image (premarket)
INSERT INTO premarket_info (
  id, creator_id, creator_address, bc_address, data_uri,
  name, description, symbol, telegram, twitter, web_site,
  premarket_goal_pers, premarket_goal_sol_lamp, premarket_deadline, premarket_created,
  state, mint_address, premarket_finished
)
SELECT gen_random_uuid(), tc.c2_id, tc.c2_addr, tc.c2_addr, 'ipfs://fake-D2',
       'F-About-No-Image', 'fake: community about here', 'FD2',
       NULL, NULL, 'https://example.com',
       4, 5 * c.lamports_per_sol, t.future_deadline, t.created_epoch,
       'premarket', tc.c2_addr, NULL
FROM tmp_time t
CROSS JOIN tmp_creators tc
CROSS JOIN tmp_const c;

-- D3 no about + image (premarket)
INSERT INTO premarket_info (
  id, creator_id, creator_address, bc_address, data_uri,
  name, description, symbol, image_url, telegram, twitter, web_site,
  premarket_goal_pers, premarket_goal_sol_lamp, premarket_deadline, premarket_created,
  state, mint_address, premarket_finished
)
SELECT gen_random_uuid(), tc.c3_id, tc.c3_addr, tc.c3_addr, 'ipfs://fake-D3',
       'F-NoAbout-With-Image', 'fake', 'FD3',
       'https://images.unsplash.com/photo-1496307042754-b4aa456c4a2d', NULL, NULL, 'https://example.com',
       3, 5 * c.lamports_per_sol, t.future_deadline, t.created_epoch,
       'premarket', tc.c3_addr, NULL
FROM tmp_time t
CROSS JOIN tmp_creators tc
CROSS JOIN tmp_const c;

-- D4 no about + no image (premarket)
INSERT INTO premarket_info (
  id, creator_id, creator_address, bc_address, data_uri,
  name, description, symbol, telegram, twitter, web_site,
  premarket_goal_pers, premarket_goal_sol_lamp, premarket_deadline, premarket_created,
  state, mint_address, premarket_finished
)
SELECT gen_random_uuid(), tc.c1_id, tc.c1_addr, tc.c1_addr, 'ipfs://fake-D4',
       'F-NoAbout-No-Image', 'fake', 'FD4',
       NULL, NULL, 'https://example.com',
       2, 5 * c.lamports_per_sol, t.future_deadline, t.created_epoch,
       'premarket', tc.c1_addr, NULL
FROM tmp_time t
CROSS JOIN tmp_creators tc
CROSS JOIN tmp_const c;

-- D5 explicit no image (premarket)
INSERT INTO premarket_info (
  id, creator_id, creator_address, bc_address, data_uri,
  name, description, symbol, telegram, twitter, web_site,
  premarket_goal_pers, premarket_goal_sol_lamp, premarket_deadline, premarket_created,
  state, mint_address, premarket_finished
)
SELECT gen_random_uuid(), tc.c2_id, tc.c2_addr, tc.c2_addr, 'ipfs://fake-D5',
       'F-No-Image-Explicit', 'fake: no image_url', 'FD5',
       'https://t.me/your_channel', NULL, 'https://example.com',
       3, 8 * c.lamports_per_sol, t.future_deadline, t.created_epoch,
       'premarket', tc.c2_addr, NULL
FROM tmp_time t
CROSS JOIN tmp_creators tc
CROSS JOIN tmp_const c;

-- ================== 7) helper-функция для holders =====================
CREATE OR REPLACE FUNCTION tmp_add_holder(
  pmid uuid,
  uid uuid,
  waddr text,
  amount_sol numeric,
  pos integer,
  make_out boolean DEFAULT false
) RETURNS void
LANGUAGE plpgsql
AS $$
DECLARE
  LAMPS bigint := (SELECT lamports_per_sol FROM tmp_const);
  created_epoch bigint := (SELECT created_epoch FROM tmp_time);
BEGIN
  INSERT INTO premarket_holders (
    id, premarket_info_id, holder_id, holder_wallet,
    amount_lamport, join_timestamp, out_timestamp
  )
  VALUES (
    gen_random_uuid(), pmid, uid, waddr,
    (amount_sol * LAMPS)::bigint,
    created_epoch + pos*10,
    CASE WHEN make_out THEN created_epoch + pos*10 + 3600 ELSE NULL END
  );
END;
$$;

-- ================== 8) HOLDERS =======================================
DO $$
DECLARE
  c1_id uuid := (SELECT c1_id FROM tmp_creators);
  c2_id uuid := (SELECT c2_id FROM tmp_creators);
  c3_id uuid := (SELECT c3_id FROM tmp_creators);

  c1_addr text := (SELECT c1_addr FROM tmp_creators);
  c2_addr text := (SELECT c2_addr FROM tmp_creators);
  c3_addr text := (SELECT c3_addr FROM tmp_creators);

  u_ids uuid[];
  len int;
  pmid uuid;
  i int;
  waddr text;
BEGIN
  SELECT array_agg(u.id ORDER BY u.id) INTO u_ids
  FROM users u WHERE u.username NOT IN ('creator_1','creator_2','creator_3');

  len := COALESCE(array_length(u_ids,1),0);
  IF len = 0 THEN RAISE EXCEPTION 'No non-creator users found'; END IF;

  -- A1
  SELECT id INTO pmid FROM premarket_info WHERE name='F-Overdue-17-not-reached' LIMIT 1;
  IF pmid IS NOT NULL THEN
    PERFORM tmp_add_holder(pmid, c1_id, c1_addr, 0.5, 1);
    PERFORM tmp_add_holder(pmid, u_ids[((2-1)%len)+1], c1_addr, 0.2, 2);
    PERFORM tmp_add_holder(pmid, u_ids[((3-1)%len)+1], c2_addr, 0.1, 3);
    PERFORM tmp_add_holder(pmid, u_ids[((4-1)%len)+1], c3_addr, 0.3, 4);
    PERFORM tmp_add_holder(pmid, u_ids[((5-1)%len)+1], c1_addr, 0.15, 5);
    PERFORM tmp_add_holder(pmid, c2_id, c2_addr, 2.0, 6);
    PERFORM tmp_add_holder(pmid, u_ids[((7-1)%len)+1], c1_addr, 0.2, 7);
    PERFORM tmp_add_holder(pmid, u_ids[((8-1)%len)+1], c2_addr, 0.25, 8);
    PERFORM tmp_add_holder(pmid, u_ids[((9-1)%len)+1], c3_addr, 0.05, 9);
    PERFORM tmp_add_holder(pmid, c3_id, c3_addr, 1.0, 10);
    PERFORM tmp_add_holder(pmid, u_ids[((11-1)%len)+1], c1_addr, 0.1, 11);
    PERFORM tmp_add_holder(pmid, u_ids[((12-1)%len)+1], c2_addr, 0.2, 12);
    PERFORM tmp_add_holder(pmid, u_ids[((13-1)%len)+1], c3_addr, 0.1, 13);
    PERFORM tmp_add_holder(pmid, u_ids[((14-1)%len)+1], c1_addr, 0.05, 14);
    PERFORM tmp_add_holder(pmid, u_ids[((15-1)%len)+1], c2_addr, 0.2, 15);
    PERFORM tmp_add_holder(pmid, u_ids[((16-1)%len)+1], c3_addr, 0.1, 16);
    PERFORM tmp_add_holder(pmid, u_ids[((17-1)%len)+1], c1_addr, 0.05, 17);
  END IF;

  -- A2
  SELECT id INTO pmid FROM premarket_info WHERE name='F-Overdue-26-reached' LIMIT 1;
  IF pmid IS NOT NULL THEN
    PERFORM tmp_add_holder(pmid, c1_id, c1_addr, 1.0, 1);
    PERFORM tmp_add_holder(pmid, c2_id, c2_addr, 5.0, 2);
    PERFORM tmp_add_holder(pmid, c3_id, c3_addr, 1.0, 3);
    FOR i IN 4..26 LOOP
      waddr := CASE WHEN (i % 3)=1 THEN c1_addr WHEN (i % 3)=2 THEN c2_addr ELSE c3_addr END;
      PERFORM tmp_add_holder(pmid, u_ids[((i-1)%len)+1], waddr, ((i % 5)*0.1 + 0.05), i);
    END LOOP;
  END IF;

  -- A3
  SELECT id INTO pmid FROM premarket_info WHERE name='F-Overdue-11-expired' LIMIT 1;
  IF pmid IS NOT NULL THEN
    FOR i IN 1..11 LOOP
      waddr := CASE WHEN (i % 3)=1 THEN c1_addr WHEN (i % 3)=2 THEN c2_addr ELSE c3_addr END;
      PERFORM tmp_add_holder(pmid, u_ids[((i-1)%len)+1], waddr, 0.1, i, true);
    END LOOP;
  END IF;

  -- A4
  SELECT id INTO pmid FROM premarket_info WHERE name='F-Overdue-28-open' LIMIT 1;
  IF pmid IS NOT NULL THEN
    FOR i IN 1..28 LOOP
      waddr := CASE WHEN (i % 3)=1 THEN c1_addr WHEN (i % 3)=2 THEN c2_addr ELSE c3_addr END;
      PERFORM tmp_add_holder(pmid, u_ids[((i-1)%len)+1], waddr, ((i % 7)*0.1 + 0.05), i);
    END LOOP;
  END IF;

  -- B1
  SELECT id INTO pmid FROM premarket_info WHERE name='F-Future-1-not-reached' LIMIT 1;
  IF pmid IS NOT NULL THEN
    PERFORM tmp_add_holder(pmid, c1_id, c1_addr, 1.0, 1);
  END IF;

  -- B2
  SELECT id INTO pmid FROM premarket_info WHERE name='F-Future-1-reached' LIMIT 1;
  IF pmid IS NOT NULL THEN
    PERFORM tmp_add_holder(pmid, c2_id, c2_addr, 3.5, 1);
  END IF;

  -- B3
  SELECT id INTO pmid FROM premarket_info WHERE name='F-Future-19-not-reached' LIMIT 1;
  IF pmid IS NOT NULL THEN
    FOR i IN 1..19 LOOP
      waddr := CASE WHEN (i % 3)=1 THEN c1_addr WHEN (i % 3)=2 THEN c2_addr ELSE c3_addr END;
      PERFORM tmp_add_holder(pmid, u_ids[((i-1)%len)+1], waddr, ((i % 6)*0.1 + 0.05), i);
    END LOOP;
  END IF;

  -- B4
  SELECT id INTO pmid FROM premarket_info WHERE name='F-Future-27-reached' LIMIT 1;
  IF pmid IS NOT NULL THEN
    PERFORM tmp_add_holder(pmid, c1_id, c1_addr, 4.0, 1);
    PERFORM tmp_add_holder(pmid, c2_id, c2_addr, 3.0, 2);
    PERFORM tmp_add_holder(pmid, c3_id, c3_addr, 2.0, 3);
    FOR i IN 4..27 LOOP
      waddr := CASE WHEN (i % 3)=1 THEN c1_addr WHEN (i % 3)=2 THEN c2_addr ELSE c3_addr END;
      PERFORM tmp_add_holder(pmid, u_ids[((i-1)%len)+1], waddr, ((i % 5)*0.1 + 0.05), i);
    END LOOP;
  END IF;

  -- C1
  SELECT id INTO pmid FROM premarket_info WHERE name='F-Launched-25-reached' LIMIT 1;
  IF pmid IS NOT NULL THEN
    PERFORM tmp_add_holder(pmid, c2_id, c2_addr, 5.0, 1);
    PERFORM tmp_add_holder(pmid, c1_id, c1_addr, 3.0, 2);
    PERFORM tmp_add_holder(pmid, c3_id, c3_addr, 2.0, 3);
    FOR i IN 4..25 LOOP
      waddr := CASE WHEN (i % 3)=1 THEN c1_addr WHEN (i % 3)=2 THEN c2_addr ELSE c3_addr END;
      PERFORM tmp_add_holder(pmid, u_ids[((i-1)%len)+1], waddr, ((i % 6)*0.15 + 0.05), i);
    END LOOP;
  END IF;

  -- D*
  SELECT id INTO pmid FROM premarket_info WHERE name='F-About-With-Image' LIMIT 1;
  IF pmid IS NOT NULL THEN
    FOR i IN 1..5 LOOP PERFORM tmp_add_holder(pmid, u_ids[((i-1)%len)+1], c1_addr, 0.2, i); END LOOP;
  END IF;

  SELECT id INTO pmid FROM premarket_info WHERE name='F-About-No-Image' LIMIT 1;
  IF pmid IS NOT NULL THEN
    FOR i IN 1..4 LOOP PERFORM tmp_add_holder(pmid, u_ids[((i-1)%len)+1], c2_addr, 0.15, i); END LOOP;
  END IF;

  SELECT id INTO pmid FROM premarket_info WHERE name='F-NoAbout-With-Image' LIMIT 1;
  IF pmid IS NOT NULL THEN
    FOR i IN 1..3 LOOP PERFORM tmp_add_holder(pmid, u_ids[((i-1)%len)+1], c3_addr, 0.1, i); END LOOP;
  END IF;

  SELECT id INTO pmid FROM premarket_info WHERE name='F-NoAbout-No-Image' LIMIT 1;
  IF pmid IS NOT NULL THEN
    FOR i IN 1..2 LOOP PERFORM tmp_add_holder(pmid, u_ids[((i-1)%len)+1], c1_addr, 0.1, i); END LOOP;
  END IF;

  SELECT id INTO pmid FROM premarket_info WHERE name='F-No-Image-Explicit' LIMIT 1;
  IF pmid IS NOT NULL THEN
    FOR i IN 1..3 LOOP PERFORM tmp_add_holder(pmid, u_ids[((i-1)%len)+1], c2_addr, 0.12, i); END LOOP;
  END IF;

END $$;

-- ================== 9) Санити: bc_address == creator_address ==========
UPDATE premarket_info
SET bc_address = creator_address
WHERE bc_address IS DISTINCT FROM creator_address;

-- ================== 10) finished => mint_address не пустой ============
UPDATE premarket_info
SET mint_address = creator_address
WHERE state = 'finished' AND (mint_address IS NULL OR mint_address = '');

COMMIT;
