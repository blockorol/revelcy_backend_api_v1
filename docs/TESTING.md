# Testing

## Common Commands

```bash
cargo fmt --check
cargo check
cargo clippy -- -D warnings
cargo test
```

Current note: `cargo fmt --check` is useful, but existing Rust code is not fully rustfmt-clean yet. CI runs formatting as advisory/non-blocking until the codebase is formatted in a dedicated cleanup.

Documentation structure can be checked with:

```powershell
.\scripts\check-agent-docs.ps1
```

## Test Assets

SQL helpers live in `test/`:

- `set_mock_for_test.sql`
- `remove_all.sql`

Inspect these scripts before using them against a local database.

## Suggested Coverage Areas

- Auth and wallet signature failure cases.
- JWT-protected route behavior.
- Premarket lifecycle transitions.
- Whitelist application/approval/rejection.
- Vesting withdrawal behavior.
- SQLx repository behavior around migrations.
- Solana transaction construction with deterministic inputs.

## External Services

Prefer mocks or isolated test inputs for:

- Solana RPC.
- IPFS/proxy calls.
- Pyth/price data.

## CI

GitHub Actions includes:

- `.github/workflows/ci.yml`: Rust checks. The format step is currently advisory/non-blocking.
- `.github/workflows/docs.yml`: documentation structure check.
- `.github/workflows/security.yml`: cargo audit and cargo deny. This workflow is currently advisory/non-blocking.
