# Quick Start

This is the short boot sequence for agents entering the repository.

## First Reads

1. `AGENTS.md`
2. `docs/agents/README.md`
3. The task-specific files listed in `docs/agents/README.md`

For most code changes, also read:

- `PROJECT_MAP.md`
- `ARCHITECTURE.md`
- `INVARIANTS.md`
- `WORKFLOWS.md`
- `CHANGE_PROTOCOL.md`

## First Files To Inspect

- Entrypoint: `src/main.rs`
- Scope registration: `src/server/mod.rs`
- Main premarket handlers: `src/server/premarket_server.rs`
- Main premarket service: `src/services/premarket_service.rs`
- Main premarket repository: `src/storage/premarket_repo.rs`
- Main premarket domain models: `src/models/premarket.rs`

## Core Commands

Run from the repository root.

```powershell
cargo fmt --check
cargo check
cargo clippy -- -D warnings
cargo test
```

For documentation-only changes:

```powershell
git diff --check
```

## Quick Classification

- Route/API change: read `API_CONTRACTS.md` and `FRONTEND_BACKEND_CONTRACT.md`.
- Business logic change: read `DOMAIN.md`, `GLOSSARY.md`, and `INVARIANTS.md`.
- Database change: read `DATABASE.md` and `MAPPING_GUIDE.md`.
- Solana change: read `SOLANA.md` and `RISK_REGISTER.md`.
- Auth/security change: read `AUTH_SECURITY.md` and `ERROR_HANDLING.md`.
- Dependency change: read `DEPENDENCIES.md`.
- Review request: read `PR_REVIEW_CHECKLIST.md`.

## Default Agent Posture

- Keep changes narrow.
- Preserve existing user work.
- Respect the three interface model levels.
- Update agent docs when the change affects future navigation, safety, or workflow.
- Report what was and was not verified.

