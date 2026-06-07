---
name: revelcy-db-change-review
description: Review Revelcy backend database changes including migrations, SQLx queries, storage models, repository behavior, cascade/delete semantics, and API/internal/storage model mapping. Use when Codex changes or reviews migrations, src/storage, database-facing service code, or schema-driven API behavior.
---

# Revelcy DB Change Review

Use this skill for schema and persistence safety.

## Workflow

1. Read `docs/agents/DATABASE.md`.
2. Read `docs/agents/MAPPING_GUIDE.md`.
3. Read `docs/agents/INVARIANTS.md`.
4. Inspect changed files in `migrations/`, `src/storage/`, `src/models/`, and affected services.

## Review Questions

- Was a new migration added instead of editing old history?
- Do SQLx row models match the schema?
- Do repository queries match nullability and column names?
- Are delete/cascade semantics intentional?
- Is data mapped through internal/domain models before API exposure?
- Did public API shape change intentionally?
- Are database-backed checks or manual schema checks needed?

Update `DATABASE.md`, `PROJECT_MAP.md`, and `MAPPING_GUIDE.md` when persistence behavior changes.

