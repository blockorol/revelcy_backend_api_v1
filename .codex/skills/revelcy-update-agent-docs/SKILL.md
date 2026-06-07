---
name: revelcy-update-agent-docs
description: Update Revelcy backend agent-facing and human-facing documentation after code, route, schema, domain, Solana, auth, dependency, testing, workflow, deployment, or public contract changes. Use when Codex changes this repository in a way that affects future agent navigation, safety rules, model boundaries, verification expectations, frontend-sensitive backend contracts, or developer docs.
---

# Revelcy Update Agent Docs

Use this skill after meaningful repository changes to keep `AGENTS.md`, `docs/agents/*`, and human-facing documentation accurate.

## Workflow

1. Read `AGENTS.md` and `docs/agents/README.md`.
2. Inspect the changed files.
3. Use `docs/agents/CHANGE_PROTOCOL.md` to decide which agent docs and human docs need updates.
4. Use `docs/agents/HUMAN_DOCS.md` for human documentation ownership.
5. Keep agent docs path-oriented and keep human docs practical for developers.
6. Run:

```powershell
git diff --check
.\scripts\check-agent-docs.ps1
```

## Common Updates

- Routes or handlers: update `PROJECT_MAP.md`, `ROUTE_HANDLERS.md`, `API_CONTRACTS.md`, and possibly `FRONTEND_BACKEND_CONTRACT.md`.
- Human-facing behavior: update the relevant files listed in `docs/agents/HUMAN_DOCS.md`.
- API DTOs or errors: update `API_CONTRACTS.md` and `ERROR_HANDLING.md`.
- Model boundary changes: update `ARCHITECTURE.md`, `MAPPING_GUIDE.md`, and `INVARIANTS.md`.
- Migrations/storage: update `DATABASE.md`.
- Solana: update `SOLANA.md` and `RISK_REGISTER.md`.
- Auth/security: update `AUTH_SECURITY.md`, `ERROR_HANDLING.md`, and `INVARIANTS.md`.
- Dependencies: update `DEPENDENCIES.md`.
- Tests/checks: update `TESTING.md`.

Do not add human-facing product docs unless the user asks for them.
