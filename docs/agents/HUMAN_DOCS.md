# Human Documentation

This file tells agents how to maintain human-facing documentation. Human docs are for developers, contributors, operators, and future open-source readers. Agent docs stay in `docs/agents`.

## Human-Facing Files

- `README.md`: project overview, quick start, links.
- `docs/README.md`: documentation index.
- `docs/DEVELOPMENT.md`: local setup and common commands.
- `docs/CONFIGURATION.md`: environment variables.
- `docs/API.md`: public API overview.
- `docs/API_CHANGELOG.md`: API changes and frontend impact.
- `docs/ARCHITECTURE.md`: human-readable architecture.
- `docs/DATABASE.md`: database and migration guide.
- `docs/SOLANA.md`: Solana integration guide.
- `docs/AUTH.md`: auth, wallet, and JWT guide.
- `docs/TESTING.md`: testing guide.
- `docs/DEPLOYMENT.md`: Docker/Railway deployment notes.
- `docs/TROUBLESHOOTING.md`: common problems.
- `docs/GLOSSARY.md`: human glossary.
- `docs/OPEN_SOURCE_CHECKLIST.md`: open-source readiness checklist.
- `CONTRIBUTING.md`: contribution guide.
- `SECURITY.md`: security reporting and sensitive-area guidance.
- `CHANGELOG.md`: project change history.
- `.github/pull_request_template.md`: PR checklist.
- `.github/ISSUE_TEMPLATE/*`: issue templates.

## Update Rules

- API route/DTO/error changes: update `docs/API.md` and `docs/API_CHANGELOG.md`.
- Env var changes: update `docs/CONFIGURATION.md` and possibly `README.md` or `docs/DEVELOPMENT.md`.
- Database or migration changes: update `docs/DATABASE.md`.
- Auth/wallet/JWT changes: update `docs/AUTH.md` and possibly `SECURITY.md`.
- Solana transaction flow changes: update `docs/SOLANA.md`, `docs/API.md`, and `docs/API_CHANGELOG.md`.
- Test command, fixture, or strategy changes: update `docs/TESTING.md` and possibly `CONTRIBUTING.md`.
- Deployment changes: update `docs/DEPLOYMENT.md` and `docs/TROUBLESHOOTING.md`.
- Domain terminology changes: update `docs/GLOSSARY.md`.
- Contributor process changes: update `CONTRIBUTING.md` and `.github/pull_request_template.md`.

## Style Rules

- Write for humans, not agents.
- Keep examples secret-free.
- Prefer concise, practical setup and maintenance guidance.
- Do not duplicate full agent-only safety checklists in human docs.
- Link to agent docs only when the content is explicitly for Codex/agents.
