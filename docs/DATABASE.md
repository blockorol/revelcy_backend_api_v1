# Database

Database schema changes are managed through SQL migration files in `migrations/`.

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
