# Project Map

This map tells agents where to look before editing.

## Runtime Entrypoints

- `src/main.rs`: binary entrypoint for `revelcy-backend-api`; loads env, creates the Postgres pool and Solana RPC client, registers Actix scopes, and binds the server.
- `src/lib.rs`: library root exposing project modules.
- `src/bin/pump_keys_generator.rs`: additional binary using `DATABASE_URL` and optional `TARGET_SUFFIX` with default `pump`.

## Top-Level Directories

- `src/`: Rust application code.
- `migrations/`: SQL schema migrations.
- `idl/`: contract/interface artifacts.
- `test/`: SQL fixtures and cleanup helpers.
- `storage/`: local runtime storage; ignored by git.
- `target/`: Cargo build output; ignored by git.

## Source Modules

### `src/server/`

Actix HTTP layer: scopes, handlers, extractors, validation, and request orchestration.

- `mod.rs`: registers public scopes in `init_servers`.
- `public_server.rs`: `/auth` routes for session, login confirmation, wallet info, and premarket info.
- `user_server.rs`: `/user` routes for profile, invite, username, avatar, list, and search behavior.
- `file_server.rs`: `/files` routes for upload and image/avatar/community image retrieval.
- `premarket_server.rs`: `/premarket` routes; largest handler file and main premarket API entrypoint.
- `premarket_validation.rs`: validation helpers for premarket requests.
- `auth_validation.rs`: auth-related validation helpers.
- `user_server_extractor.rs`: user extraction helper.
- `proxy.rs`: `/proxy` routes.
- `whitelist_handlers.rs`: whitelist handler logic used by premarket routes.
- `premarket_getter_handler.rs`: read-oriented premarket handler helpers.
- `vesing_server_handler.rs`: vesting handler logic. Keep the current filename unless doing an explicit rename task.

### `src/api/`

API-facing DTO and error layer.

- `dto.rs`: request/response DTOs for auth and public API responses.
- `premarket.rs`: premarket API types.
- `user.rs`: user API types.
- `vesting.rs`: vesting API types.
- `whitelist.rs`: whitelist API types.
- `errors.rs`: API error mapping.
- `mod.rs`: module declarations.

### `src/services/`

Business logic and external integrations.

- `auth_service.rs`: session/auth confirmation logic.
- `jwt_service.rs`: JWT creation/validation helpers.
- `wallet_service.rs`: wallet/signature-related logic.
- `user_service.rs`: user-level business logic.
- `user_info_service.rs`: additional user profile info.
- `premarket_service.rs`: main premarket domain service.
- `whitelist_service.rs`: whitelist business rules.
- `vesting_service.rs`: vesting business rules.
- `background_finaliser.rs`: background finalization behavior.
- `file_service.rs`: local file storage helpers.
- `ipfs_service.rs`: IPFS upload/proxy logic.
- `solana_price_service.rs`: Solana/Pyth price-related logic.
- `solana_service.rs`: older/general Solana helpers.
- `solana_service_v2/`: current Solana transaction-building modules.

### `src/services/solana_service_v2/`

High-risk Solana transaction area.

- `mod.rs`: module exports.
- `env.rs`: Solana env and network resolution helpers.
- `constants.rs`: chain/program constants.
- `contract_specific.rs`: program-specific account/data helpers.
- `solana_methods.rs`: generic Solana helper methods.
- `utils.rs`: shared transaction utilities.
- `tx_create_premarket.rs`: create premarket transaction.
- `tx_join_premarket.rs`: join transaction.
- `tx_out_premarket.rs`: out/leave transaction.
- `tx_finish_premarket.rs`: finish transaction.
- `tx_kill_premarket.rs`: kill/cancel transaction.
- `tx_extend_premarket.rs`: extend deadline transaction.
- `tx_update_uri.rs`: update metadata URI transaction.
- `tx_update_premarket_data.rs`: update premarket data transaction.
- `tx_claim_tokens.rs`: claim tokens transaction.
- `tx_withdraw_vesting.rs`: vesting withdrawal transaction.
- `vesting.rs`: vesting-specific Solana helpers.

### `src/storage/`

SQLx database access layer.

- `models.rs`: database row models.
- `premarket_repo.rs`: premarket, community, holder, state, links, URI, and claim persistence.
- `user_repo.rs`: user persistence.
- `user_info_repo.rs`: additional user info persistence.
- `whitelist_repo.rs`: whitelist persistence.
- `vesting_repo.rs`: vesting persistence.
- `signing_keys.rs`: signing key persistence.
- `mod.rs`: module declarations.

### `src/models/`

Domain/service models.

- `premarket.rs`: premarket params, state, token/community models, holder models, on-chain data models, and Pyth data models.
- `user.rs`: user domain models.
- `vesting.rs`: vesting domain models.
- `whitelist.rs`: whitelist domain models.
- `mod.rs`: module declarations.

### `src/middleware/`

Request middleware.

- `cors.rs`: CORS origins from `CORS_ORIGINS`.
- `jwt.rs`: JWT middleware.
- `mod.rs`: module declarations.

### `src/config/`

Environment accessors, env-name constants, defaults, and startup validation. Some defaults are development placeholders. Do not document or add real secrets in tracked files.

- `mod.rs`: public config facade, re-exports section accessors, and `validate_startup_config`.
- `env.rs`: low-level `std::env` access helpers used only by config modules.
- `database.rs`: database env accessors.
- `server.rs`: host, port, and CORS env accessors.
- `security.rs`: JWT and Revelcy signer secret accessors.
- `pyth.rs`: Pyth integration env accessors.
- `storage.rs`: local storage env accessors.
- `solana.rs`: Solana RPC, network, and program id env accessors.
- `pump_keys.rs`: pump key generator env accessors.

## Route Map

Registered in `src/server/mod.rs`:

- `/auth`
- `/user`
- `/files`
- `/premarket`
- `/proxy`

### `/auth`

- `GET /auth/start_session`
- `POST /auth/confirm_login`
- `GET /auth/wallet_info/{pubkey}`
- `GET /auth/premarket_info/{premarket_account}`

### `/user`

- `POST /user/set_additional_info`
- `POST /user/set_invite_code`
- `POST /user/update_username`
- `POST /user/update_avatar`
- `POST /user/short_list`
- `POST /user/search`

### `/files`

- `POST /files/upload/{name}`
- `GET /files/image/{name}`
- `GET /files/avatar/{user_id}`
- `GET /files/community_image/{token_id}`

### `/premarket`

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

### `/proxy`

- `POST /proxy/pump_ipfs`

## Migrations

Migration files live in `migrations/` and are sequential SQL files.

Current schema themes:

- Initial schema.
- Premarket finish/deadline changes.
- Holder claim and extension fields.
- Hide/short link fields.
- Additional user info.
- Invite codes and cascade behavior.
- Whitelist table and user whitelist fields.
- User state and username index.
- Vesting tables.
- Concept visibility and creation timestamp.

