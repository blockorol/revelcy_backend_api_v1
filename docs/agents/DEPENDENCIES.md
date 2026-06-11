# Dependencies

This file records dependency constraints and upgrade cautions for agents.

## Rust Toolchain

- Rust edition: `2021`.
- Toolchain file: `.rust-toolchain.toml`, currently Rust `1.87`.
- Main manifest: `Cargo.toml`.
- Lockfile: `Cargo.lock`.
- `.github/workflows/security.yml` pins `cargo-audit` to `0.22.1` because newer `0.22.x` releases require Rust `1.88` or newer.

Do not update the Rust toolchain or lockfile during unrelated work. Solana-related crates are sensitive to Rust/toolchain upgrades.

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

## Skill Validation Dependency

The system skill validator at `C:\Users\wking.korol\.codex\skills\.system\skill-creator\scripts\quick_validate.py` requires Python `yaml` support. If it is unavailable locally, run the repository-level structural check instead:

```powershell
.\scripts\check-agent-docs.ps1
```

## Repository Automation

Dependency and security automation lives in:

- `.github/dependabot.yml`
- `.github/workflows/security.yml`
- `deny.toml`

CI lives in:

- `.github/workflows/ci.yml`

Update human-facing `docs/DEPENDENCIES.md` when dependency constraints or automation expectations change.

License checks are intentionally not enforced yet. Do not add a license policy until the project chooses one.
