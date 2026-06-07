# Auth And Security

This file is for agents changing auth, JWT, wallet signature validation, secrets, file upload, or proxy behavior.

## Main Files

- `src/server/public_server.rs`: public auth/session routes.
- `src/server/auth_validation.rs`: auth validation helpers.
- `src/middleware/jwt.rs`: JWT middleware.
- `src/services/auth_service.rs`: auth/session logic.
- `src/services/wallet_service.rs`: wallet/signature logic.
- `src/services/jwt_service.rs`: JWT helpers.
- `src/middleware/cors.rs`: CORS config.
- `src/server/file_server.rs`: file routes.
- `src/services/file_service.rs`: file storage helpers.
- `src/server/proxy.rs`: proxy routes.
- `src/services/ipfs_service.rs`: IPFS/proxy integration.

## Security Rules

- Do not bypass wallet signature validation.
- Do not weaken JWT validation for protected behavior.
- Do not trust user identity until middleware/extractor/auth checks have run.
- Do not expose internal errors, secrets, RPC credentials, private keys, or raw config in public responses.
- Do not add real secret values to tracked files.
- Do not log private keys, JWT secrets, seed material, or signed payloads.

## Mutating Endpoint Rule

Endpoints that mutate user, premarket, whitelist, vesting, claim, file, or proxy-related state should validate:

- Authenticated user context.
- Ownership or permission.
- Request validity.
- Domain state preconditions.

## File And Proxy Rule

For file/proxy changes, check:

- Path traversal.
- Storage directory confinement.
- Upload content assumptions.
- Public response shape.
- Secret-bearing upstream requests.
- Whether the route should require auth.

## CORS Rule

CORS behavior is controlled by `CORS_ORIGINS` in `src/middleware/cors.rs`. Do not broaden origins casually.

