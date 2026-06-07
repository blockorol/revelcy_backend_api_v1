# Testing

This file tells agents what to run and how to think about verification.

## Common Commands

Run from the repository root.

```powershell
cargo fmt --check
cargo check
cargo clippy -- -D warnings
cargo test
```

Current note: `cargo fmt --check` reports many existing formatting diffs. CI keeps formatting advisory/non-blocking until a dedicated rustfmt cleanup is performed.

For documentation-only changes:

```powershell
git diff --check
.\scripts\check-agent-docs.ps1
```

CI runs Rust checks through `.github/workflows/ci.yml` and documentation structure checks through `.github/workflows/docs.yml`.

## Current Test Assets

- `test/set_mock_for_test.sql`: SQL fixture/setup helper.
- `test/remove_all.sql`: SQL cleanup helper.

Before using these scripts, inspect them and confirm they match the current schema.

## Verification By Change Type

For documentation-only changes:

- Run `git diff --check`.
- Run `.\scripts\check-agent-docs.ps1` when agent docs or project skills changed.
- Confirm links and filenames are consistent.

For route/API changes:

- Run `cargo check`.
- Run focused tests if they exist.
- Consider adding tests for changed request/response behavior.

For database changes:

- Check new migration order.
- Check affected SQLx queries against the schema.
- Run database-backed tests if a local database is available.

For Solana transaction changes:

- Prefer tests around deterministic transaction construction.
- Mock or isolate external RPC where possible.
- Verify network and signer assumptions manually if automated coverage is missing.

For auth/security changes:

- Run focused auth/JWT/wallet tests if they exist.
- Add tests for bypass and failure cases when feasible.
- Manually review `AUTH_SECURITY.md` and `INVARIANTS.md`.

## Reporting

In the final response, say which checks ran and which did not. Do not claim runtime behavior was tested if only docs or static checks were run.
