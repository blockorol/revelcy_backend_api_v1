# Contributing

Thanks for considering a contribution.

## Development

Read [Development](docs/DEVELOPMENT.md) before changing code.

Common checks:

```bash
cargo fmt --check
cargo check
cargo clippy -- -D warnings
cargo test
```

## Pull Requests

Include:

- Summary of changes.
- Tests/checks run.
- API changes.
- Database migrations.
- Solana/security impact.
- Documentation updates.

## Code Guidelines

- Keep HTTP contracts in `src/api`.
- Keep internal/domain models in `src/models`.
- Keep database row/query models in `src/storage`.
- Do not edit old migrations for normal feature work.
- Treat auth, wallet signatures, Solana transaction builders, and file/proxy behavior as security-sensitive.

## Documentation

Update human docs when developer-facing behavior changes. Agent documentation in `docs/agents` explains the documentation update rules used by Codex.
