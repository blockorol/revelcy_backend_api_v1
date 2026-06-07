---
name: revelcy-api-contract-review
description: Review Revelcy backend route, handler, DTO, API error, auth requirement, and frontend-sensitive contract changes. Use when Codex modifies or reviews files under src/server or src/api, public route behavior, request/response shapes, or backend behavior consumed by the frontend.
---

# Revelcy API Contract Review

Use this skill to protect public HTTP contracts.

## Workflow

1. Read `docs/agents/API_CONTRACTS.md`.
2. Read `docs/agents/ROUTE_HANDLERS.md`.
3. Read `docs/agents/FRONTEND_BACKEND_CONTRACT.md`.
4. Inspect changed `src/server/*` and `src/api/*` files.
5. Check whether any route, method, request field, response field, auth requirement, or error shape changed.

## Review Questions

- Is the route path or HTTP method stable?
- Is the handler still the intended owner?
- Are API DTOs still external-only?
- Are internal/domain models used before service/storage work?
- Are storage rows kept out of public responses?
- Is frontend impact documented or mentioned?
- Is the change intentionally breaking?

Update agent docs through `$revelcy-update-agent-docs` if route or contract documentation changed.

