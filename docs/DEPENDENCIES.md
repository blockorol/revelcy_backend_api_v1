# Dependencies

This project uses Rust dependencies from `Cargo.toml` and locked versions in `Cargo.lock`.

## Main Groups

- Web: `actix-web`, `actix-cors`, `actix-multipart`, `actix-service`
- Database: `sqlx`
- Async runtime: `tokio`
- Auth/security: `jsonwebtoken`, `sha2`, `base64`, `rand`
- Solana: `solana-client`, `solana-sdk`, `solana-transaction-status`, `spl-token`, `spl-associated-token-account`
- HTTP clients: `reqwest`, `awc`
- Serialization: `serde`, `serde_json`, `borsh`, `bincode`
- Logging/tracing: `tracing`, `tracing-subscriber`, `log`
- Environment: `dotenvy`, `dotenv`

## Known Constraint

The project README historically notes that `sqlx 0.7` conflicts with `solana-sdk 1.17`.

Be careful when upgrading:

- SQLx.
- Solana crates.
- JWT/crypto-related crates.
- HTTP/TLS features.
- Runtime-related crates.

## Upgrade Checklist

- Update `Cargo.toml` intentionally.
- Review `Cargo.lock`.
- Run `cargo check`.
- Run relevant tests.
- Review Docker build impact.
- Update this document if a new constraint is discovered.

## Automation

The repository includes:

- `.github/dependabot.yml`
- `.github/workflows/security.yml`
- `deny.toml`

These help surface dependency updates, advisories, and license/security concerns.

