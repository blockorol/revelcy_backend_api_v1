# API Overview

This document summarizes public API areas. It is not a generated OpenAPI specification.

## Route Groups

### Auth

- `GET /auth/start_session`
- `POST /auth/confirm_login`
- `GET /auth/wallet_info/{pubkey}`
- `GET /auth/premarket_info/{premarket_account}`

Auth uses wallet signature confirmation and JWT-related behavior. See [Auth](AUTH.md).

### User

- `POST /user/set_additional_info`
- `POST /user/set_invite_code`
- `POST /user/update_username`
- `POST /user/update_avatar`
- `POST /user/short_list`
- `POST /user/search`

### Files

- `POST /files/upload/{name}`
- `GET /files/image/{name}`
- `GET /files/avatar/{user_id}`
- `GET /files/community_image/{token_id}`

### Premarket

Read/update routes:

- `GET /premarket/get_user_entry`
- `GET /premarket/get_main_info`
- `GET /premarket/get_list`
- `GET /premarket/get_dynamic_info`
- `GET /premarket/get_holder_entry_price`
- `POST /premarket/update_community`
- `POST /premarket/update_availability`
- `POST /premarket/vesting/update_info`

Whitelist routes:

- `POST /premarket/whitelist/add_user`
- `POST /premarket/whitelist/add_user_list`
- `POST /premarket/whitelist/apply`
- `POST /premarket/whitelist/get`
- `POST /premarket/whitelist/remove_user`
- `POST /premarket/whitelist/approve`
- `POST /premarket/whitelist/reject`

Concept routes:

- `POST /premarket/concept/create`
- `GET /premarket/concept/get`

Transaction routes:

- `POST /premarket/tx/create`
- `POST /premarket/tx/join`
- `POST /premarket/tx/out`
- `POST /premarket/tx/finish`
- `POST /premarket/tx/kill`
- `POST /premarket/tx/extend_premarket`
- `POST /premarket/tx/update_uri`
- `POST /premarket/tx/claim_tokens`
- `POST /premarket/tx/withdraw_vesting`
- `POST /premarket/tx/sign_and_send_transaction`

### Proxy

- `POST /proxy/pump_ipfs`

## Contract Notes

Route paths, methods, request fields, response fields, auth requirements, and error shapes are frontend-sensitive. Breaking API changes should be recorded in [API changelog](API_CHANGELOG.md).

Request and response examples live in [API examples](API_EXAMPLES.md). The source of truth for DTO fields is `src/api`.
