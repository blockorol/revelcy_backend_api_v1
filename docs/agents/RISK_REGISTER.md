# Risk Register

This file lists areas where agents should slow down and run an extra review pass.

## Very High Risk

### `src/services/solana_service_v2/*`

Funds-sensitive transaction builders and chain behavior.

Risks:

- Wrong account derivation.
- Wrong signer assumptions.
- Wrong program constants.
- Wrong network selection.
- Transaction output no longer matching frontend/API expectations.

Read before changing:

- `SOLANA.md`
- `INVARIANTS.md`
- `DOMAIN.md`

### Auth, JWT, And Wallet Signature Flow

Files:

- `src/server/public_server.rs`
- `src/server/auth_validation.rs`
- `src/middleware/jwt.rs`
- `src/services/auth_service.rs`
- `src/services/wallet_service.rs`
- `src/services/jwt_service.rs`

Risks:

- Auth bypass.
- JWT accepted with wrong claims.
- Wallet signature validation weakened.
- Sensitive data leaked through errors or logs.

Read before changing:

- `AUTH_SECURITY.md`
- `ERROR_HANDLING.md`
- `INVARIANTS.md`

### Migrations And Cascade Behavior

Files:

- `migrations/*`
- `src/storage/*_repo.rs`
- `src/storage/models.rs`

Risks:

- Breaking existing data.
- Incorrect cascade/delete semantics.
- SQLx model/query mismatch.
- Public API shape changed unintentionally through schema changes.

Read before changing:

- `DATABASE.md`
- `API_CONTRACTS.md`
- `INVARIANTS.md`

## High Risk

### `src/server/premarket_server.rs`

Large route/handler file for the main domain.

Risks:

- Mixing API, internal, and storage model levels further.
- Duplicating validation or business logic.
- Changing public routes accidentally.
- Missing related whitelist/vesting/transaction behavior.

Read before changing:

- `PROJECT_MAP.md`
- `API_CONTRACTS.md`
- `DOMAIN.md`
- `WORKFLOWS.md`

### Premarket State And Holder Flows

Files:

- `src/services/premarket_service.rs`
- `src/storage/premarket_repo.rs`
- `src/models/premarket.rs`

Risks:

- Database state diverges from intended chain-side behavior.
- Claim/out/join behavior becomes inconsistent.
- List/get dynamic info becomes inconsistent with stored state.

### File Upload, Local Storage, Proxy, IPFS

Files:

- `src/server/file_server.rs`
- `src/services/file_service.rs`
- `src/server/proxy.rs`
- `src/services/ipfs_service.rs`

Risks:

- Path traversal.
- Secret leakage to client.
- Unprotected mutation/upload path.
- Unexpected public access to local files.

Read before changing:

- `AUTH_SECURITY.md`
- `ERROR_HANDLING.md`

## Medium Risk

### `src/config/mod.rs`

Risks:

- Secret-like defaults.
- Missing startup validation.
- Network/config mismatch.

### Deployment Files

Files:

- `Dockerfile`
- `Dockerfile.pump_gen`
- `Dockerfile_old`
- `docker-compose.yml`
- `docker-compose.pupm_gen.yml`
- `entrypoint.sh`
- `railway.toml`
- `railway_key_gen.toml`

Risks:

- Changing production build/runtime behavior during unrelated work.
- Breaking migrations at startup.

Do not change deployment files unless the user asks for deployment work.

