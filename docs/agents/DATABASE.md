# Database

This file is for agents changing migrations, SQLx repositories, or storage models.

## Interface Level Rule

`src/storage/*` is the storage interface level. It should contain database row/query shapes and repository functions only.

Do not pass storage models directly to API responses. Do not use API DTOs as SQLx row models. Map storage rows into internal/domain models in `src/models` before service logic returns data upward.

## Files

- `migrations/`: ordered SQL migration files.
- `src/storage/models.rs`: database row models.
- `src/storage/premarket_repo.rs`: premarket, community, holder, state, links, URI, and claim persistence.
- `src/storage/user_repo.rs`: user persistence.
- `src/storage/user_info_repo.rs`: additional user info persistence.
- `src/storage/whitelist_repo.rs`: whitelist persistence.
- `src/storage/vesting_repo.rs`: vesting persistence.
- `src/storage/signing_keys.rs`: signing key persistence.

## Migration Rules

- Add a new migration for normal schema changes.
- Do not edit old migrations unless the user explicitly asks for a local-only migration rewrite.
- Keep migration numbering sequential.
- Check cascade behavior before changing deletes or foreign keys.
- Update storage models and repository queries with schema changes.
- Update API models only when external contract changes.

## Current Schema Themes

- Initial schema.
- Premarket finish/deadline changes.
- Holder claim and extension fields.
- Hide/short link fields.
- Additional user info.
- Invite codes and cascade behavior.
- Whitelist table and user whitelist fields.
- User state and username index.
- Vesting tables.
- Concept visibility and creation timestamp.

## Repository Pattern

Preferred flow:

1. Repository function receives explicit arguments or internal/domain model data.
2. SQLx query maps rows into storage model or local query result.
3. Repository returns storage result or maps into an internal/domain model if that is the established local pattern.
4. Service layer owns business interpretation.

Avoid route-specific response shaping in repository functions.

## Schema Change Checklist

When changing the database:

- Add migration.
- Update `src/storage/models.rs`.
- Update affected `src/storage/*_repo.rs`.
- Update `src/models/*` if internal service data shape changes.
- Update `src/api/*` only if public request/response shape changes.
- Update `docs/agents/DATABASE.md` and possibly `PROJECT_MAP.md`.

