# Agent Documentation

This directory is for Codex and other code agents. It is not product documentation and should stay operational, concise, and path-oriented.

Before editing unfamiliar code, read the files that match the task.

## Reading Guide

- Use `PROJECT_MAP.md` to answer: where is the code?
- Use `ARCHITECTURE.md` to answer: how do the layers fit together?
- Use `DOMAIN.md` to answer: what does this business concept mean?
- Use `INVARIANTS.md` to answer: what must not be broken?
- Use `API_CONTRACTS.md` to answer: what external HTTP/API shapes must stay stable?
- Use `DATABASE.md` to answer: how does persistence work?
- Use `SOLANA.md` to answer: how do chain transaction flows work?
- Use `AUTH_SECURITY.md` to answer: what security-sensitive rules apply?
- Use `TESTING.md` to answer: what should be checked for this change?
- Use `WORKFLOWS.md` to answer: how should this kind of change be made?
- Use `CHANGE_PROTOCOL.md` to answer: what must be checked and documented after a change?

## Task Shortcuts

For endpoint or route changes, read:

- `PROJECT_MAP.md`
- `ARCHITECTURE.md`
- `API_CONTRACTS.md`
- `WORKFLOWS.md`
- `CHANGE_PROTOCOL.md`

For business logic changes, read:

- `DOMAIN.md`
- `INVARIANTS.md`
- `WORKFLOWS.md`
- `CHANGE_PROTOCOL.md`

For database changes, read:

- `PROJECT_MAP.md`
- `ARCHITECTURE.md`
- `DATABASE.md`
- `WORKFLOWS.md`
- `CHANGE_PROTOCOL.md`

For Solana transaction changes, read:

- `ARCHITECTURE.md`
- `DOMAIN.md`
- `INVARIANTS.md`
- `SOLANA.md`
- `WORKFLOWS.md`

For auth, JWT, wallet, file upload, proxy, or secret-handling changes, read:

- `ARCHITECTURE.md`
- `AUTH_SECURITY.md`
- `INVARIANTS.md`
- `CHANGE_PROTOCOL.md`

For test or verification changes, read:

- `TESTING.md`
- `CHANGE_PROTOCOL.md`

## Interface Model Levels

This project intends to keep three separate model/interface levels:

- API models in `src/api`: external HTTP request/response contracts only.
- Internal/domain models in `src/models`: data passed inside services and integrations.
- Storage models in `src/storage`: database row/query shapes only.

Some current code may violate this separation. Preserve the intended direction in new changes and avoid spreading existing violations.

## Documentation Rules

- Write for agents, not end users.
- Prefer exact paths and module names.
- Do not include secrets or real local environment values.
- Do not duplicate large code snippets.
- Update these files when routes, architecture, domain behavior, invariants, workflows, or required checks change.
