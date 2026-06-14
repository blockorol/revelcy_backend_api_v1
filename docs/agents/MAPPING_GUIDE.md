# Mapping Guide

This guide explains how data should move across the three model/interface levels.

## Intended Levels

- API level: `src/api/*`
- Internal/domain level: `src/models/*`
- Storage level: `src/storage/*`

Some existing code may violate this. New work should move toward explicit boundaries.

## Preferred Flow

For incoming requests:

1. Handler receives API DTO from `src/api`.
2. Handler validates HTTP/auth concerns.
3. Handler or nearby mapper converts API DTO into internal/domain model from `src/models`.
4. Service uses internal/domain model.
5. Repository receives explicit arguments or storage-oriented data.
6. Repository maps SQL rows into storage models.
7. Service maps storage data into internal/domain result.
8. Handler maps internal/domain result into API response DTO.

## Allowed Directions

Good:

- `src/server` uses `src/api` for request/response and `src/models` for service calls.
- `src/services` uses `src/models` and calls `src/storage`.
- `src/storage` uses storage models and returns persistence results.
- Repository functions that cross into `src/services` may return internal/domain models after mapping SQL rows inside `src/storage`.
- `src/api` defines external shapes without importing storage rows.

Avoid:

- `src/storage` depending on API DTOs.
- Public handlers returning storage rows directly.
- Service functions requiring API request DTOs when an internal model would be clearer.
- Domain models depending on Actix request/response types.

## Mapping Placement

Place mapping where the boundary is crossed:

- API DTO -> internal model: handler or small helper near handler/API module.
- Internal model -> storage query args: service or repository boundary.
- Storage model -> internal model: repository return mapping or service mapping, following local pattern.
- Internal model -> API DTO: handler or API response helper.

If a mapping becomes reused across multiple handlers, create a small local helper. Avoid a broad abstraction until repetition is real.

## Schema Field Changes

When a database field is added:

1. Add migration.
2. Update storage model/query.
3. Decide if internal model needs the field.
4. Decide if API response/request needs the field.
5. Update docs for each affected level.

Do not expose a DB field in the API just because it exists in storage.

## Refactoring Existing Violations

When cleaning an existing model-level violation:

- Keep the refactor scoped.
- Preserve public API behavior unless explicitly changing it.
- Add or update tests where behavior could shift.
- Update `LEGACY_NOTES.md` if the violation is resolved or if a new known violation is discovered.

