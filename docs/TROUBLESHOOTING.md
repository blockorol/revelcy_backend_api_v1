# Troubleshooting

## Missing `DATABASE_URL`

The server requires `DATABASE_URL` at startup. Set it in your local environment or ignored `.env` file.

## Missing `SOLANA_RPC`

The server requires `SOLANA_RPC` at startup. Use a devnet or mainnet RPC endpoint appropriate for your environment.

## Missing Startup Config

The API server validates `DATABASE_URL`, `SOLANA_RPC`, `JWT_SECRET`, `CURRENT_HOST`, `PYTH_MAINNET_URL`, `REVELCY_AUTH_PRIVATE_KEY_DEV`, and `REVELCY_AUTH_PRIVATE_KEY_MAIN` after loading `.env`.

The `pump-key-generator` worker only validates `DATABASE_URL`; `TARGET_SUFFIX` defaults to `pump`.

## CORS Issues

Check `CORS_ORIGINS`. CORS behavior is implemented in `src/middleware/cors.rs`.

## SQLx Or Solana Dependency Conflicts

The root README notes a conflict between `sqlx 0.7` and `solana-sdk 1.17`. Avoid broad dependency upgrades without checking compile behavior.

## Docker Build Issues

The main Dockerfile builds static OpenSSL and a musl target. Failures may involve OpenSSL, musl, Cargo dependencies, or network access during image build.

## File Storage Issues

Check `STORAGE_DIR` and local filesystem permissions. File routes are under `/files`.

## Solana RPC Errors

Check:

- RPC URL and network.
- `NETWORK`.
- Devnet/mainnet variables.
- Rate limits or upstream RPC failures.
