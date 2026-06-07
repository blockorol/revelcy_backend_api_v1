# Release Process

Use this checklist before publishing or deploying a release.

## Pre-Release Checks

```bash
cargo fmt --check
cargo check
cargo clippy -- -D warnings
cargo test
```

For documentation and agent support:

```powershell
.\scripts\check-agent-docs.ps1
```

## Review Areas

- API changes recorded in `docs/API_CHANGELOG.md`.
- Database migrations reviewed.
- Solana transaction changes reviewed.
- Auth/security-sensitive changes reviewed.
- Human docs updated.
- Agent docs updated.
- Changelog updated.

## Dependency Review

For dependency changes:

- Review `Cargo.toml` and `Cargo.lock`.
- Read `docs/DEPENDENCIES.md`.
- Run dependency/security automation where available.

## Deployment Review

- Confirm deployment env vars.
- Confirm migration behavior.
- Confirm no secrets are tracked.
- Confirm Docker/Railway files still match the intended deployment path.

## Post-Release

- Smoke test auth/session flow.
- Smoke test a read-only premarket endpoint.
- Check logs for startup, migration, database, and RPC errors.

