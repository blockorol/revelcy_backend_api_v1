# Legacy Notes

This file records known imperfect or transitional areas. Do not casually "clean up" these areas during unrelated tasks.

## Three Interface Levels Are Not Fully Enforced Yet

Intended model levels:

- API models in `src/api`.
- Internal/domain models in `src/models`.
- Storage models in `src/storage`.

Some current code may mix these levels. When touching such code:

- Avoid spreading the violation.
- Prefer explicit mapping at boundaries.
- Keep refactors scoped unless the user asks for interface cleanup.

Resolved cleanup:

- `src/models/premarket.rs` no longer imports premarket API DTOs for dynamic info mapping; internal-to-API conversions for those DTOs live in `src/api/premarket.rs`.
- `src/services/user_info_service.rs` no longer imports API errors; it returns a service-owned error that `src/server/user_server.rs` maps to `ApiError`.
- `src/services/user_service.rs` no longer imports Actix HTTP errors; it returns `UserServiceError` that handlers map at the server/API boundary.
- `src/services/whitelist_service.rs` no longer imports Actix HTTP errors; it returns `WhitelistServiceError` that `src/server/whitelist_handlers.rs` maps to `ApiError`.
- `src/services/ipfs_service.rs` no longer imports Actix HTTP errors; it returns `IpfsServiceError` for callers to map at the API/server boundary when re-enabled.
- `src/services/vesting_service.rs` no longer imports Actix HTTP errors; it returns `VestingServiceError` that `src/server/vesing_server_handler.rs` and premarket transaction handlers map at the server/API boundary.
- `src/services/premarket_service.rs` no longer imports `src/storage/models.rs` row structs; `src/storage/premarket_repo.rs` maps premarket storage rows to internal models before returning to services.
- `src/storage/vesting_repo.rs` and `src/storage/signing_keys.rs` no longer expose storage row structs in their public return types; they map rows to internal models before returning to services.

## `vesing_server_handler.rs` Typo

The file name `src/server/vesing_server_handler.rs` appears to be a typo. Keep the current name unless the user explicitly asks for rename/cleanup. Renaming may require module and import updates.

## `solana_service.rs` And `solana_service_v2`

Both exist:

- `src/services/solana_service.rs`
- `src/services/solana_service_v2/`

Treat `solana_service_v2` as the current transaction-builder area, but inspect call sites before moving logic. Do not delete or merge legacy Solana helpers during unrelated work.

## Docker And Deployment File History

Multiple Docker/Railway-related files exist. Some are specialized:

- `Dockerfile`
- `Dockerfile.pump_gen`
- `docker-compose.yml`
- `docker-compose.pupm_gen.yml`
- `railway.toml`
- `railway_key_gen.toml`

Do not remove or rename them unless the task is explicitly about deployment cleanup.

## Minimal Human README

The root `README.md` is currently minimal. This agent documentation is not a replacement for human-facing open-source documentation.

Do not expand human docs unless the user asks for human-facing documentation.
