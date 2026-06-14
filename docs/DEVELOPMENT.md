# Development

This guide is for developers working on the Revelcy backend.

## Prerequisites

- Rust toolchain from `.rust-toolchain.toml`
- Postgres
- Cargo
- Access to a Solana RPC endpoint

Docker files are available, but local Rust development usually starts with Cargo.

## Local Setup

Create local environment variables outside git-tracked files. At minimum, set:

- `DATABASE_URL`
- `SOLANA_RPC`
- `JWT_SECRET`
- `CURRENT_HOST`
- `PYTH_MAINNET_URL`
- `REVELCY_AUTH_PRIVATE_KEY_DEV`
- `REVELCY_AUTH_PRIVATE_KEY_MAIN`

See [Configuration](CONFIGURATION.md) for the full list.

## Run Locally

```bash
cargo run --bin revelcy-backend-api
```

The service binds to `PORT` or defaults to `8080`.
Runtime logs use `tracing`; set `RUST_LOG` when you need a different filter.

## Common Commands

```bash
cargo fmt --check
cargo check
cargo clippy -- -D warnings
cargo test
```

Additional binary:

```bash
cargo run --bin pump-key-generator
```

## Project Structure

- `src/main.rs`: server entrypoint.
- `src/server`: Actix scopes and handlers.
- `src/api`: API request/response DTOs.
- `src/models`: internal/domain models.
- `src/services`: business logic and integrations.
- `src/storage`: SQLx repositories and database row models.
- `src/services/solana_service_v2`: Solana transaction builders.
- `migrations`: database migrations.

## Model Levels

The codebase intends to keep three interface levels:

- API models in `src/api`: external HTTP contracts.
- Internal/domain models in `src/models`: service and integration data.
- Storage models in `src/storage`: database row/query shapes.

Some existing code may still mix these levels. New work should move toward explicit mapping between them.

## Migrations

Add a new migration under `migrations/` for normal schema changes. Do not edit old migrations unless you are intentionally rewriting local development history.
