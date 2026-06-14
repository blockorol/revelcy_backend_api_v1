# Local Database

This guide covers local Postgres setup notes for backend development.

## Required Variable

Set `DATABASE_URL` to a Postgres database that the backend can read and write.

Example shape:

```bash
DATABASE_URL=postgres://postgres:postgres@localhost:5432/revelcy
```

Do not commit real local credentials.

## Migrations

Migration files live in `migrations/`.

Deployment applies migrations with Goose through `entrypoint.sh`:

```sh
goose -dir /migrations postgres "$DATABASE_URL" up
```

For local migration work, install Goose or run the same migration flow inside the Docker image. Before applying migrations locally, inspect:

- `migrations/`
- `entrypoint.sh`
- `Dockerfile`

## Local Reset Helpers

SQL helper scripts live in `test/`:

- `test/set_mock_for_test.sql`
- `test/remove_all.sql`

Inspect these scripts before running them. They are development/test helpers and may remove data.

## Schema Change Checklist

When changing local schema:

- Add a new migration.
- Update `src/storage/models.rs`.
- Update affected repository queries.
- Update `docs/DATABASE.md` if the schema area changes.
- Update `docs/API.md` and `docs/API_EXAMPLES.md` only if public API behavior changes.
