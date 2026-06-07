---
name: revelcy-update-agent-docs
description: Update Revelcy backend agent-facing documentation after code, route, schema, domain, Solana, auth, dependency, testing, or workflow changes. Use when Codex changes this repository in a way that affects future agent navigation, safety rules, model boundaries, verification expectations, or frontend-sensitive backend contracts.
---

# Revelcy Update Agent Docs

Use this skill after meaningful repository changes to keep `AGENTS.md` and `docs/agents/*` accurate.

## Workflow

1. Read `AGENTS.md` and `docs/agents/README.md`.
2. Inspect the changed files.
3. Use `docs/agents/CHANGE_PROTOCOL.md` to decide which agent docs need updates.
4. Keep updates path-oriented and agent-facing.
5. Run:

```powershell
git diff --check
.\scripts\check-agent-docs.ps1
```

## Common Updates

- Routes or handlers: update `PROJECT_MAP.md`, `ROUTE_HANDLERS.md`, `API_CONTRACTS.md`, and possibly `FRONTEND_BACKEND_CONTRACT.md`.
- API DTOs or errors: update `API_CONTRACTS.md` and `ERROR_HANDLING.md`.
- Model boundary changes: update `ARCHITECTURE.md`, `MAPPING_GUIDE.md`, and `INVARIANTS.md`.
- Migrations/storage: update `DATABASE.md`.
- Solana: update `SOLANA.md` and `RISK_REGISTER.md`.
- Auth/security: update `AUTH_SECURITY.md`, `ERROR_HANDLING.md`, and `INVARIANTS.md`.
- Dependencies: update `DEPENDENCIES.md`.
- Tests/checks: update `TESTING.md`.

Do not add human-facing product docs unless the user asks for them.

