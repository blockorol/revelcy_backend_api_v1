# Code Style

This file captures local code placement and style rules for agents. Follow existing patterns first.

## Placement Rules

- HTTP route registration and handlers belong in `src/server`.
- API request/response contracts belong in `src/api`.
- Internal/domain data belongs in `src/models`.
- Business decisions belong in `src/services`.
- SQL and row mapping belong in `src/storage`.
- Cross-cutting request logic belongs in `src/middleware`.
- Environment accessors belong in `src/config` or existing env helper modules.
- Solana transaction construction belongs in `src/services/solana_service_v2`.

## Three Interface Levels

Keep model levels separate:

- API DTOs: external HTTP only.
- Internal/domain models: service and integration flow.
- Storage models: SQLx/database only.

Do not introduce new code that passes API DTOs directly into storage or returns storage rows directly from handlers.

## Function Shape

- Prefer small helper functions when validation or mapping logic repeats.
- Keep route handlers focused on extraction, validation, service calls, and response mapping.
- Keep repository functions focused on a single persistence action or query.
- Keep Solana builder functions explicit about network, accounts, signer assumptions, and returned transaction shape.

## Naming

- Follow existing module naming unless the task is explicitly a rename/cleanup.
- Keep existing public route names stable unless the user asks for a breaking API change.
- Do not rename `vesing_server_handler.rs` casually; see `LEGACY_NOTES.md`.

## Comments

- Add comments only when they clarify non-obvious domain, SQL, Solana, or security behavior.
- Avoid comments that restate the code.
- Prefer documenting recurring rules in `docs/agents` rather than scattering long comments through code.

## Error Handling

- Map internal errors to API responses intentionally.
- Do not leak secrets or raw internal details through public errors.
- See `ERROR_HANDLING.md`.

## Dependencies

- Do not update dependencies during unrelated work.
- Read `DEPENDENCIES.md` before version or feature changes.

## Formatting

Rust formatting is configured by `rustfmt.toml`. Run `cargo fmt --check` before finalizing Rust code changes.

Current state: existing code is not fully rustfmt-clean. Do not format the whole codebase during unrelated work; do that only as a dedicated cleanup.
