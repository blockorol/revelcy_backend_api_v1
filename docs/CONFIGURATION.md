# Configuration

Configuration is read from environment variables. Local `.env` files are intentionally ignored by git.

## Required

- `DATABASE_URL`: Postgres connection string.
- `SOLANA_RPC`: default Solana RPC endpoint used at server startup.

## Common Optional Variables

- `PORT`: HTTP port, default `8080`.
- `CURRENT_HOST`: base host used by config helpers, default `http://localhost:8080/`.
- `JWT_SECRET`: JWT signing/validation secret. The code has a development fallback; production must set a real secret.
- `CORS_ORIGINS`: allowed CORS origins.
- `NETWORK`: network selector used by Solana helpers, default empty string in `src/config/mod.rs`.
- `STORAGE_DIR`: local file storage directory.
- `SOLANA_DEVNET_RPC`: devnet RPC endpoint.
- `SOLANA_MAINNET_RPC`: mainnet RPC endpoint.
- `PYTH_SUBDOMAIN`: Pyth subdomain, default `api`.
- `PYTH_SECRET_TOKEN`: Pyth secret token, default empty string.
- `PYTH_MAINNET_URL`: Pyth mainnet URL, default empty string.
- `REVELCY_AUTH_PRIVATE_KEY`: private key used by auth/Solana flows, default empty string.
- `TARGET_SUFFIX`: suffix used by the `pump-key-generator` binary.

## Where Variables Are Read

- `src/main.rs`: `DATABASE_URL`, `SOLANA_RPC`, `PORT`.
- `src/config/mod.rs`: host, JWT, Pyth, auth private key, network.
- `src/middleware/cors.rs`: `CORS_ORIGINS`.
- `src/services/file_service.rs`: `STORAGE_DIR`.
- `src/services/solana_service.rs`: devnet/mainnet RPC defaults.
- `src/services/solana_service_v2/env.rs`: network-specific RPC/env behavior.
- `src/bin/pump_keys_generator.rs`: `DATABASE_URL`, `TARGET_SUFFIX`.

## Security Notes

- Do not commit real secrets.
- Do not log private keys, JWT secrets, RPC credentials, seed material, or signed sensitive payloads.
- Review [Security](../SECURITY.md) before changing auth, wallet, file upload, proxy, or Solana transaction behavior.
