# Invariants

Preserve these constraints unless the user explicitly asks to change the behavior. If code contradicts this file, inspect the code and update this file only when the actual intended behavior is clear.

## Repository Safety

- Do not commit secrets, keys, RPC credentials, JWT secrets, or local environment files.
- Do not rewrite existing migrations as part of normal feature work.
- Do not change deployment files for unrelated application changes.
- Do not rename public routes, request fields, response fields, or database columns without treating it as a breaking change.

## Layering

- Server handlers should orchestrate HTTP concerns and call services.
- Services should own business decisions.
- Storage repositories should own SQL and row mapping.
- Domain models in `src/models` should not know about Actix request/response behavior.
- API DTOs should not own persistence behavior.
- API DTOs from `src/api` should not be used as storage row models.
- Storage models from `src/storage` should not be returned directly as public API responses.
- Internal/domain models from `src/models` should be the preferred data shape across service boundaries.
- Keep mappings between API, internal, and storage model levels explicit.
- Solana transaction builders should not be mixed into handler code if a service/builder already exists.

## Authentication And Authorization

- JWT validation must remain required for protected behavior.
- Wallet signature validation must not be bypassed.
- User identity from middleware/extractors should be trusted only after the relevant auth checks.
- Endpoints that mutate user, premarket, whitelist, vesting, or claim state should validate ownership/permissions in the appropriate handler/service path.

## Premarket State

- State transitions should be explicit and persisted through `premarket_repo`.
- Holder join/out/claim operations should keep database state consistent with the intended chain-side state.
- Deadline/finish/extend behavior should keep naming and semantics consistent across database, service, API, and transaction code.
- Hidden/visible and concept visibility fields should be handled intentionally in list/get endpoints.

## Whitelist

- Whitelist add/remove/apply/approve/reject behavior should keep user and premarket context aligned.
- Bulk whitelist updates should preserve validation behavior for individual entries unless explicitly changed.

## Vesting

- Vesting API, DB rows, and Solana withdrawal transaction behavior must remain aligned.
- Vesting schedule or withdrawal changes should be reflected in service logic and transaction builders together.

## Solana

- Network selection must be explicit and use the existing env/config helpers.
- Program constants, PDA/account derivation, signer handling, and instruction data are sensitive.
- Transaction builders should return deterministic transaction data for the same inputs where possible.
- Do not silently switch devnet/mainnet behavior.
- Do not log private keys, signatures, seed material, or raw secret config.

## Database

- SQLx query changes should be checked against the current schema.
- Additive schema changes usually need row model, repository, service, and API review.
- Deletes can be hard or soft depending on existing behavior. Preserve existing semantics unless the task says otherwise.
- Cascade behavior exists in migrations; check it before changing delete/update flows.

## Error Handling

- Preserve API error shape unless intentionally changing API behavior.
- Avoid leaking internal errors, secrets, private config, or raw chain internals in public responses.
- Prefer contextual errors in services/repositories when it helps debugging without exposing sensitive data externally.

## Files And Proxy

- File paths should stay constrained to intended storage locations.
- Upload handlers should avoid path traversal.
- Proxy/IPFS helpers should not expose internal credentials.
