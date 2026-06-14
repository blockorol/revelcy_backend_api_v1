# API Changelog

Track public API changes here, especially changes that may affect frontend clients.

## Unreleased

- Initial API changelog added.
- `POST /premarket/tx/kill`: error responses now use the structured `ApiError` JSON shape instead of ad hoc plain-text bodies for validation/auth/build failures. Frontend clients should read `error`, `code`, and optional field/message details.
- `POST /premarket/concept/create`: negative `goal_sol_lamp` now returns a structured validation error instead of risking a request panic.

When adding entries, include:

- Route or DTO affected.
- Breaking or non-breaking.
- Frontend impact.
- Migration or rollout notes if needed.
