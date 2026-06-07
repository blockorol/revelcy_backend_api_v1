# Testing

## Common Commands

```bash
cargo fmt --check
cargo check
cargo clippy -- -D warnings
cargo test
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
