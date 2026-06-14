# Architecture

The backend uses a layered Actix/SQLx service architecture with Solana integration modules.

## Startup

`src/main.rs`:

1. Loads environment variables.
2. Creates a SQLx Postgres pool from `DATABASE_URL`.
3. Creates a shared Solana RPC client from `SOLANA_RPC`.
4. Builds the Actix app.
5. Registers middleware and route scopes.
6. Binds to `PORT` or `8080`.

## Request Flow

1. `src/server/mod.rs` registers route scopes.
2. A handler in `src/server/*` extracts and validates the request.
3. API DTOs from `src/api` represent external input/output.
4. Services in `src/services` apply business logic.
5. Repositories in `src/storage` perform SQLx queries.
6. Handlers map service results into API responses.

## Interface Levels

The intended design has three model levels:

- API: `src/api`, external HTTP contracts.
- Internal/domain: `src/models`, data passed inside the backend.
- Storage: `src/storage`, database row/query shapes.

Mapping between levels should be explicit.

## Main Domains

- Auth and wallet login.
- Users and profiles.
- Premarket lifecycle.
- Whitelist.
- Vesting.
- Files/IPFS/proxy behavior.
- Solana transaction construction.
