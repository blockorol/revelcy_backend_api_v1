# Route Handlers

This file maps public routes to handler functions. Use it when changing routing, API DTOs, frontend contracts, or handler ownership.

Keep this file aligned with `src/server/mod.rs` and the relevant `src/server/*_server.rs` files.

## Registered Scopes

Registered in `src/server/mod.rs`:

- `/auth`: `public_server::public_scope()`
- `/user`: `user_server::user_scope()`
- `/files`: `file_server::file_scope()`
- `/premarket`: `premarket_server::pub_scope()`
- `/proxy`: `proxy::proxy_scope()`

## `/auth`

Defined in `src/server/public_server.rs`.

- `GET /auth/start_session` -> `start_session`
- `POST /auth/confirm_login` -> `confirm_login`
- `GET /auth/wallet_info/{pubkey}` -> `wallet_info`
- `GET /auth/premarket_info/{premarket_account}` -> `premarket_info`

## `/user`

Defined in `src/server/user_server.rs`.

- `POST /user/set_additional_info` -> `user_set_info`
- `POST /user/set_invite_code` -> `set_invite_code`
- `POST /user/update_username` -> `update_username`
- `POST /user/update_avatar` -> `update_avatar`
- `POST /user/short_list` -> `get_users_short_list`
- `POST /user/search` -> `search_users`

## `/files`

Defined in `src/server/file_server.rs`.

- `POST /files/upload/{name}` -> `upload_png`
- `GET /files/image/{name}` -> `get_png`
- `GET /files/avatar/{user_id}` -> `get_avatar`
- `GET /files/community_image/{token_id}` -> `get_community_image`

## `/premarket`

Defined in `src/server/premarket_server.rs`.

Read/update routes:

- `GET /premarket/get_user_entry` -> `get_user_entry`
- `GET /premarket/get_main_info` -> `get_main_info`
- `GET /premarket/get_list` -> `get_list_main_info`
- `GET /premarket/get_dynamic_info` -> `get_dynamic_info`
- `GET /premarket/get_holder_entry_price` -> `get_holder_entry_price`
- `POST /premarket/update_community` -> `update_community_info`
- `POST /premarket/update_availability` -> `update_availability`
- `POST /premarket/vesting/update_info` -> `update_vesting`

Whitelist routes:

- `POST /premarket/whitelist/add_user` -> `add_whitelist_user`
- `POST /premarket/whitelist/add_user_list` -> `add_whitelist_user_list`
- `POST /premarket/whitelist/apply` -> `apply_whitelist`
- `POST /premarket/whitelist/get` -> `get_premarket_whitelist`
- `POST /premarket/whitelist/remove_user` -> `remove_whitelist_user`
- `POST /premarket/whitelist/approve` -> `whitelist_approve`
- `POST /premarket/whitelist/reject` -> `whitelist_reject`

Concept routes:

- `POST /premarket/concept/create` -> `create_concept`
- `GET /premarket/concept/get` -> `get_user_concept`

Transaction routes:

- `POST /premarket/tx/create` -> `create_premarket_tx`
- `POST /premarket/tx/join` -> `join_premarket_tx`
- `POST /premarket/tx/out` -> `out_premarket_tx`
- `POST /premarket/tx/finish` -> `finish_premarket_tx`
- `POST /premarket/tx/kill` -> `kill_premarket_tx`
- `POST /premarket/tx/extend_premarket` -> `extend_premarket_tx`
- `POST /premarket/tx/update_uri` -> `update_uri_tx`
- `POST /premarket/tx/claim_tokens` -> `claim_tokens_tx`
- `POST /premarket/tx/withdraw_vesting` -> `withdraw_vesting_tx`
- `POST /premarket/tx/sign_and_send_transaction` -> `sign_and_send_transaction`

## `/proxy`

Defined in `src/server/proxy.rs`.

- `POST /proxy/pump_ipfs` -> `pump_ipfs`

