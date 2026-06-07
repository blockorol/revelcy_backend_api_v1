# Dependencies

This file records dependency constraints and upgrade cautions for agents.

## Rust Toolchain

- Rust edition: `2021`.
- Toolchain file: `.rust-toolchain.toml`.
- Main manifest: `Cargo.toml`.
- Lockfile: `Cargo.lock`.

Do not update the Rust toolchain or lockfile during unrelated work.

## Important Dependencies

- `actix-web`: HTTP server framework.
- `sqlx`: Postgres access and migrations support.
- `tokio`: async runtime.
- `jsonwebtoken`: JWT handling.
- `solana-client`, `solana-sdk`, `solana-transaction-status`: Solana integration.
- `spl-associated-token-account`, `spl-token`: SPL token helpers.
- `reqwest` and `awc`: HTTP clients.
- `dotenvy` and `dotenv`: env loading appears in different binaries/modules.
- `tracing` and `tracing-subscriber`: logging/tracing.

## Known Constraint

The root `README.md` notes that `sqlx` version `0.7` has a conflict with `solana-sdk 1.17`.

Before upgrading `sqlx`, Solana crates, or transitive crypto/network dependencies:

- Inspect `Cargo.toml`.
- Inspect `Cargo.lock`.
- Run `cargo check`.
- Run relevant tests.
- Watch for compile-time feature conflicts.

## Sensitive Upgrade Areas

Treat these as high-risk dependency updates:

- Solana crates.
- SQLx.
- JWT/crypto-related crates.
- Actix middleware-related crates.
- HTTP client TLS features.

Do not run broad dependency upgrades unless the user asks for dependency maintenance.

## Docker Build Dependencies

The main `Dockerfile` builds with:

- Rust base image.
- musl target.
- static OpenSSL build.
- Goose migration tool.
- Alpine runtime with PostgreSQL client.

Dependency changes can affect Docker builds even if local `cargo check` passes.
