# Configuration

Configuration is read from environment variables. Local `.env` files are intentionally ignored by git.

## Required

- `DATABASE_URL`: Postgres connection string.
- `SOLANA_RPC`: default Solana RPC endpoint used at server startup.

## Common Optional Variables

- `PORT`: HTTP port, default `8080`.
- `CURRENT_HOST`: base host used by config helpers, default `http://localhost:8080/`.
- `JWT_SECRET`: JWT signing/validation secret.
- `CORS_ORIGINS`: allowed CORS origins.
- `NETWORK`: network selector used by Solana helpers.
- `STORAGE_DIR`: local file storage directory.
- `SOLANA_DEVNET_RPC`: devnet RPC endpoint.
- `SOLANA_MAINNET_RPC`: mainnet RPC endpoint.
- `PYTH_SUBDOMAIN`: Pyth subdomain, default `api`.
- `PYTH_SECRET_TOKEN`: Pyth secret token.
- `PYTH_MAINNET_URL`: Pyth mainnet URL.
- `REVELCY_AUTH_PRIVATE_KEY`: private key used by auth/Solana flows.
- `TARGET_SUFFIX`: suffix used by the `pump-key-generator` binary.

## Security Notes

- Do not commit real secrets.
- Do not log private keys, JWT secrets, RPC credentials, seed material, or signed sensitive payloads.
- Review [Security](../SECURITY.md) before changing auth, wallet, file upload, proxy, or Solana transaction behavior.
