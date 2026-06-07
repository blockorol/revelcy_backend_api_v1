# Agent Operating Guide

This repository is a Rust backend for Revelcy. It uses Actix Web, SQLx/Postgres, JWT-based auth, Solana wallet/signature flows, IPFS/file helpers, and Solana transaction builders.

Use this file as the first context document before making code changes. The detailed agent documentation starts at `docs/agents/README.md`.

## Core Rules

- Do not change application code when the user asks only for documentation, planning, review, or analysis.
- Preserve user changes in the working tree. Do not revert unrelated edits.
- Do not commit secrets or local environment files. `.env` and `.env.*` are intentionally ignored.
- Treat Solana transaction logic, auth, JWT, wallet signature validation, and migrations as high-risk areas.
- Do not edit existing migrations to change history. Add a new migration unless the user explicitly asks for a local-only rewrite.
- Prefer narrow changes that follow the current layering: server handlers -> services -> storage -> models/API DTOs.
- Preserve the three interface model levels: API models in `src/api` for external HTTP contracts, internal/domain models in `src/models` for in-process service flow, and storage models in `src/storage` for database rows.
- Update agent documentation when a change alters routes, module ownership, architecture, database schema, domain behavior, invariants, workflows, or required checks.

## Repository Map

- `src/main.rs`: process entrypoint; loads env, creates Postgres pool and Solana RPC client, starts Actix.
- `src/server/`: Actix scopes, handlers, extractors, request validation.
- `src/api/`: API DTOs, API-facing types, and response error mapping.
- `src/services/`: business logic and integrations.
- `src/storage/`: SQLx repository functions and database row models.
- `src/models/`: domain/service models shared across layers.
- `src/middleware/`: CORS and JWT middleware.
- `src/config/`: environment accessors.
- `src/services/solana_service_v2/`: Solana transaction-building and chain-specific helpers.
- `migrations/`: database schema changes.
- `idl/`: contract/interface artifacts.
- `test/`: SQL fixtures and cleanup helpers.
- `storage/`: local runtime storage; ignored by git.

Read `docs/agents/README.md` before changing unfamiliar modules.

## Common Commands

Run from the repository root.

```powershell
cargo fmt --check
cargo check
cargo clippy -- -D warnings
cargo test
```

Useful targeted commands:

```powershell
cargo run --bin revelcy-backend-api
cargo run --bin pump-key-generator
```

This project has Docker and Railway deployment files. Do not change deployment behavior unless the task is explicitly about deployment.

## Environment Variables

Required or commonly used variables are referenced in `src/main.rs`, `src/config/mod.rs`, `src/middleware/cors.rs`, `src/server/public_server.rs`, and Solana service modules.

Important names:

- `DATABASE_URL`
- `SOLANA_RPC`
- `SOLANA_DEVNET_RPC`
- `SOLANA_MAINNET_RPC`
- `PORT`
- `JWT_SECRET`
- `CURRENT_HOST`
- `CORS_ORIGINS`
- `NETWORK`
- `STORAGE_DIR`
- `PYTH_SUBDOMAIN`
- `PYTH_SECRET_TOKEN`
- `PYTH_MAINNET_URL`
- `REVELCY_AUTH_PRIVATE_KEY`
- `TARGET_SUFFIX`

Do not add real values to tracked files.

## Change Checklist

For route or handler changes:

- Check `src/server/mod.rs` and the relevant `*_server.rs` file.
- Check API DTOs in `src/api/`.
- Update `docs/agents/PROJECT_MAP.md` if routes or module ownership changed.

For business logic changes:

- Start in the matching `src/services/*_service.rs`.
- Check corresponding repository functions in `src/storage/`.
- Update `docs/agents/DOMAIN.md` or `docs/agents/INVARIANTS.md` if behavior changed.

For database changes:

- Add a new migration under `migrations/`.
- Update storage row models in `src/storage/models.rs` and repository queries.
- Update `docs/agents/PROJECT_MAP.md` and `docs/agents/WORKFLOWS.md` if the schema affects agent workflow.

For Solana transaction changes:

- Work primarily under `src/services/solana_service_v2/`.
- Check domain models in `src/models/premarket.rs`.
- Treat all account derivation, signer, program id, and network selection behavior as sensitive.

## Agent Documentation

- `docs/agents/README.md`: navigation for agent documentation.
- `docs/agents/PROJECT_MAP.md`: file/module ownership and route map.
- `docs/agents/ARCHITECTURE.md`: runtime shape, layers, and cross-module data flow.
- `docs/agents/DOMAIN.md`: domain concepts and behavior.
- `docs/agents/INVARIANTS.md`: constraints agents should preserve.
- `docs/agents/API_CONTRACTS.md`: API route and DTO contract rules.
- `docs/agents/ROUTE_HANDLERS.md`: route-to-handler map.
- `docs/agents/DATABASE.md`: database and repository map.
- `docs/agents/SOLANA.md`: Solana transaction and integration map.
- `docs/agents/AUTH_SECURITY.md`: auth, JWT, wallet, secret, file/proxy safety notes.
- `docs/agents/TESTING.md`: verification strategy for agents.
- `docs/agents/GLOSSARY.md`: domain and codebase terminology.
- `docs/agents/RISK_REGISTER.md`: known risky files and change areas.
- `docs/agents/LEGACY_NOTES.md`: known imperfect areas that should not be casually rewritten.
- `docs/agents/ERROR_HANDLING.md`: error mapping and public/private error rules.
- `docs/agents/DEPENDENCIES.md`: dependency constraints and upgrade cautions.
- `docs/agents/QUICK_START.md`: short boot sequence for agents.
- `docs/agents/CODE_STYLE.md`: local coding conventions and placement rules.
- `docs/agents/MAPPING_GUIDE.md`: explicit API/internal/storage model mapping guidance.
- `docs/agents/FRONTEND_BACKEND_CONTRACT.md`: frontend-sensitive backend contract notes.
- `docs/agents/PR_REVIEW_CHECKLIST.md`: review-mode checklist for agents.
- `docs/agents/HUMAN_DOCS.md`: human-facing documentation ownership and update rules.
- `docs/agents/WORKFLOWS.md`: recipes for common code changes.
- `docs/agents/CHANGE_PROTOCOL.md`: required checks and documentation update rules.
