# Architecture

This backend follows a mostly layered Actix/SQLx service architecture with Solana integration modules.

## Startup Flow

The binary starts in `src/main.rs`.

1. Load `.env` through `dotenvy`.
2. Read `DATABASE_URL`.
3. Create a SQLx Postgres pool.
4. Read `SOLANA_RPC`.
5. Create a shared nonblocking Solana RPC client.
6. Build an Actix `App`.
7. Register CORS, database pool, RPC client, and server scopes.
8. Bind to `PORT`, defaulting to `8080`.

## Request Flow

Normal HTTP request flow:

1. Route registered in `src/server/mod.rs`.
2. Handler in `src/server/*_server.rs`.
3. Validation in handler or validation helper module.
4. Business decision in `src/services/*_service.rs`.
5. Persistence through `src/storage/*_repo.rs`.
6. Response DTO or error mapping through `src/api/`.

Keep these boundaries stable unless the task explicitly asks for architectural refactoring.

## Interface Model Levels

The intended model/interface design has three levels:

- API interface: `src/api/*` is only for external HTTP contracts, request DTOs, response DTOs, and API-facing error shapes.
- Internal interface: `src/models/*` is for in-process service/domain models passed between handlers, services, Solana integrations, and other internal modules.
- Storage interface: `src/storage/models.rs` and repository-local row shapes are only for database persistence and SQLx row mapping.

Data should cross boundaries through explicit mapping:

1. HTTP request DTO from `src/api`.
2. Internal/domain model from `src/models`.
3. Storage model/query arguments in `src/storage`.
4. Internal/domain result from service logic.
5. HTTP response DTO from `src/api`.

Some current code may not fully respect this separation. When touching an area that mixes these levels, avoid making the coupling worse. Prefer small, local mapping helpers over passing API DTOs into storage or returning storage rows directly from handlers.

## Layer Responsibilities

### Server Layer

`src/server/` owns HTTP concerns:

- Actix scopes and route registration.
- Request extraction.
- Request validation.
- Calling services.
- Mapping service results into responses.

Server handlers should not own large SQL queries or Solana transaction construction details.

### API Layer

`src/api/` owns external API shapes:

- Request DTOs.
- Response DTOs.
- API-specific enums and validation-adjacent shapes.
- Error response mapping.

API DTOs should not own persistence behavior.

### Service Layer

`src/services/` owns business logic:

- Auth/session behavior.
- User/profile behavior.
- Premarket lifecycle behavior.
- Whitelist and vesting rules.
- File/IPFS/proxy integration logic.
- Solana and price integration orchestration.

Services may call storage and integration helpers. They should not depend on Actix response details.

### Storage Layer

`src/storage/` owns SQL and row mapping:

- SQLx queries.
- Database row models.
- Repository functions.

Storage functions should not own HTTP response behavior or route-level permissions.

### Domain Model Layer

`src/models/` owns business-facing structs/enums shared across services and integrations.

Use `src/models/premarket.rs` for premarket lifecycle, transaction parameter, holder, on-chain, and Pyth data models.

### Middleware Layer

`src/middleware/` owns cross-cutting request behavior:

- JWT middleware.
- CORS middleware.

## Solana Architecture

Solana behavior is split between:

- `src/services/solana_service.rs`: older/general Solana helpers.
- `src/services/solana_service_v2/`: current transaction-building modules.
- `src/models/premarket.rs`: transaction input/output and on-chain models.
- `src/services/premarket_service.rs` and `src/services/vesting_service.rs`: domain orchestration around transaction flows.

Transaction modules should keep account derivation, instruction data, program constants, network behavior, and signer assumptions explicit.

## Configuration

Environment values are read in:

- `src/main.rs`
- `src/config/mod.rs`
- `src/middleware/cors.rs`
- `src/server/public_server.rs`
- `src/services/solana_service.rs`
- `src/services/solana_service_v2/env.rs`
- `src/bin/pump_keys_generator.rs`

Important names:

- `DATABASE_URL`
- `SOLANA_RPC`
- `SOLANA_DEVNET_RPC`
- `SOLANA_MAINNET_RPC`
- `PORT`
- `JWT_SECRET`
- `CURRENT_HOST`
- `CORS_ORIGINS`
- `NETWORK`
- `STORAGE_DIR`
- `PYTH_SUBDOMAIN`
- `PYTH_SECRET_TOKEN`
- `PYTH_MAINNET_URL`
- `REVELCY_AUTH_PRIVATE_KEY`
- `TARGET_SUFFIX`

Do not add real values to tracked files.

## Deployment Shape

Deployment-related files exist at the repository root:

- `Dockerfile`
- `Dockerfile.pump_gen`
- `Dockerfile_old`
- `docker-compose.yml`
- `docker-compose.pupm_gen.yml`
- `entrypoint.sh`
- `railway.toml`
- `railway_key_gen.toml`

Do not change deployment behavior unless the task is explicitly about deployment.
