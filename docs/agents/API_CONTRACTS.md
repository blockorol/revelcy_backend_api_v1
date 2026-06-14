# API Contracts

This file is for agents changing HTTP routes, request DTOs, response DTOs, or API-visible errors.

## Interface Level Rule

`src/api/*` is the API interface level. It should contain external HTTP request/response contracts only.

Do not use API DTOs as database row models. Do not return storage models directly as API responses. Map through internal/domain models in `src/models` when data crosses into service logic.

Known intended levels:

- API models: `src/api/*`
- Internal/domain models: `src/models/*`
- Storage models: `src/storage/models.rs` and repository-local query shapes

Some existing code may mix these levels. New changes should move toward separation.

## Route Ownership

Route registration starts in `src/server/mod.rs`.

Current scopes:

- `/auth`: `src/server/public_server.rs`
- `/user`: `src/server/user_server.rs`
- `/files`: `src/server/file_server.rs`
- `/premarket`: `src/server/premarket_server.rs`
- `/proxy`: `src/server/proxy.rs`

Keep `docs/agents/PROJECT_MAP.md` aligned with route additions, removals, or renames.

## DTO Ownership

- Auth/public DTOs: `src/api/dto.rs`
- Premarket DTOs: `src/api/premarket.rs`
- User DTOs: `src/api/user.rs`
- Vesting DTOs: `src/api/vesting.rs`
- Whitelist DTOs: `src/api/whitelist.rs`
- API errors: `src/api/errors.rs`

When adding fields, decide whether the field is:

- External only: add to `src/api`.
- Internal service data: add to `src/models`.
- Database row/query data: add to `src/storage/models.rs` or repository-specific query shape.

## Breaking Change Rules

Treat these as breaking unless the user explicitly asks for them:

- Renaming routes.
- Changing HTTP methods.
- Renaming request or response fields.
- Changing field meaning or units.
- Changing auth requirements.
- Changing error status or error body shape.
- Removing response fields.
- Making optional fields required.

For intentional breaking changes, mention the API impact in the final response and update this file if the change alters recurring rules.

## Handler Pattern

Preferred flow:

1. Extract/deserialize API DTO.
2. Validate request and auth context in `src/server`.
3. Convert to internal/domain model from `src/models`.
4. Call `src/services`.
5. Convert service result into API response DTO.
6. Map errors through API/server error handling.

Avoid putting business rules into DTO conversion unless the rule is purely about API shape.

