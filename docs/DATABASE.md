# Database

Database schema changes are managed through SQL migration files in `migrations/`.

Migrations are applied with Goose in deployment. The main container copies `migrations/` into `/migrations`, and `entrypoint.sh` runs:

```sh
goose -dir /migrations postgres "$DATABASE_URL" up
```

## Repository Layer

SQLx repository modules live in `src/storage`:

- `premarket_repo.rs`
- `user_repo.rs`
- `user_info_repo.rs`
- `whitelist_repo.rs`
- `vesting_repo.rs`
- `signing_keys.rs`

Database row models live in `src/storage/models.rs`.

## Migration Policy

- Add a new migration for normal schema changes.
- Keep migration numbering sequential.
- Do not edit old migrations unless intentionally rewriting local development history.
- Keep migrations Goose-compatible.
- Update storage models and repository queries with schema changes.
- Update API docs only when external request/response behavior changes.

## Current Schema Areas

- Premarket and community data.
- Holder entries and claim status.
- Deadline/finish/extension behavior.
- User additional info and invite codes.
- Whitelist data.
- Vesting tables.
- Concept visibility and creation timestamps.

## Core Tables

The initial schema creates:

- `users`
- `wallets`
- `premarket_info`
- `community_info`
- `community_links`
- `premarket_holders`
- `signing_keys`

Later migrations add or change:

- finish/deadline fields on `premarket_info`
- claimed and extended fields
- hide/short-link fields
- additional user info
- invite codes
- cascade behavior
- whitelist tables and user whitelist state
- user state and username index
- `vesting_info`
- vesting token fields on `premarket_holders`
- concept visibility and creation timestamps

## Important Relationships

- `wallets.user_id` references `users.id`.
- `community_info.id` references `premarket_info.id`.
- `community_links.community_info_id` references `community_info.id`.
- `premarket_holders.premarket_info_id` references `premarket_info.id` with cascade delete.
- `vesting_info.premarket_id` references `premarket_info.id` with cascade delete.
