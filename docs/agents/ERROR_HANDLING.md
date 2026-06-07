# Error Handling

This file is for agents changing API errors, service errors, storage errors, or logging around failures.

## Ownership

- API error shape belongs near `src/api/errors.rs` and server handlers.
- Service errors should carry enough context for debugging without exposing secrets externally.
- Storage errors should remain close to repository functions and be mapped upward intentionally.
- Public responses should not leak raw internal errors, secret config, private keys, RPC credentials, or sensitive chain internals.

## Boundary Rule

Errors should cross the three interface levels intentionally:

1. Storage/repository error.
2. Service/domain interpretation.
3. API/server response mapping.

Do not expose storage error strings directly as public API responses unless existing behavior requires it and the content is safe.

## API Error Rules

Treat these as API contract changes:

- Status code changes.
- Error body shape changes.
- Error code/name changes.
- Changing validation failure behavior.
- Exposing or hiding fields in errors.

Update `API_CONTRACTS.md` if external error behavior changes.

## Logging Rules

Do not log:

- Private keys.
- JWT secrets.
- Seed material.
- Raw signed payloads when they are sensitive.
- RPC credentials.
- Pyth or IPFS secret tokens.

Prefer logging stable identifiers and contextual operation names over full request bodies for sensitive flows.

## Solana Error Notes

For Solana transaction failures:

- Keep enough internal context to diagnose account/network/instruction issues.
- Do not expose secret material.
- Be careful not to convert all chain errors into identical generic responses if callers need actionable failure classes.

## Validation Errors

Validation errors belong near handlers or validation helper modules. Avoid duplicating the same validation logic across handlers and services unless the current code already requires it.

