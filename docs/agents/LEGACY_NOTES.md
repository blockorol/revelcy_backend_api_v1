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
