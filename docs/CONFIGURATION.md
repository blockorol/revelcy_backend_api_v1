# Configuration

Configuration is read from environment variables. Local `.env` files are intentionally ignored by git.

## Required

For the API server:

- `DATABASE_URL`: Postgres connection string.
- `SOLANA_RPC`: default Solana RPC endpoint used at server startup.
- `JWT_SECRET`: JWT signing/validation secret.
- `CURRENT_HOST`: base host used by config helpers.
- `PYTH_MAINNET_URL`: Pyth mainnet URL.
- `REVELCY_AUTH_PRIVATE_KEY_DEV`: devnet Revelcy signer key.
- `REVELCY_AUTH_PRIVATE_KEY_MAIN`: mainnet Revelcy signer key.

For the `pump-key-generator` worker:

- `DATABASE_URL`: Postgres connection string.

## Common Optional Variables

- `PORT`: HTTP port, default `8080`.
- `CORS_ORIGINS`: allowed CORS origins.
- `NETWORK`: network selector used by Solana helpers, default empty string.
- `STORAGE_DIR`: local file storage directory.
- `SOLANA_DEVNET_RPC`: devnet RPC endpoint.
- `SOLANA_MAINNET_RPC`: mainnet RPC endpoint.
- `PYTH_SUBDOMAIN`: Pyth subdomain, default `api`.
- `PYTH_SECRET_TOKEN`: Pyth secret token, default empty string.
- `PURPLE_PROGRAM_ID_DEV`: devnet program id; code has a fallback.
- `PURPLE_PROGRAM_ID_MAIN`: mainnet program id; code has a fallback.
- `REVELCY_AUTH_PRIVATE_KEY`: legacy private key accessor, default empty string.
- `TARGET_SUFFIX`: suffix used by the `pump-key-generator` binary, default `pump`.
- `RUST_LOG`: tracing filter for local/runtime logs, default effectively `info` in the API binary.

## Where Variables Are Read

- `src/main.rs`: API startup validation, `DATABASE_URL`, `SOLANA_RPC`, `PORT`.
- `src/config/`: env names, defaults, and validators.
- `src/middleware/cors.rs`: `CORS_ORIGINS`.
- `src/services/file_service.rs`: `STORAGE_DIR`.
- `src/services/public_info_service.rs`: default `SOLANA_RPC` through the shared Solana RPC client factory.
- `src/services/solana_rpc_client.rs`: default and network-specific Solana RPC client construction.
- `src/services/solana_service.rs`: network-specific Solana RPC client construction through the shared factory.
- `src/services/solana_service_v2/env.rs`: network-specific RPC/env behavior.
- `src/bin/pump_keys_generator.rs`: pump-key validation, `DATABASE_URL`, optional `TARGET_SUFFIX`.

## Security Notes

- Do not commit real secrets.
- Do not log private keys, JWT secrets, RPC credentials, seed material, or signed sensitive payloads.
- Review [Security](../SECURITY.md) before changing auth, wallet, file upload, proxy, or Solana transaction behavior.
