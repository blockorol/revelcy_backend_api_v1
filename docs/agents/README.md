# Agent Documentation

This directory is for Codex and other code agents. It is not product documentation and should stay operational, concise, and path-oriented.

Before editing unfamiliar code, read the files that match the task.

## Reading Guide

- Use `PROJECT_MAP.md` to answer: where is the code?
- Use `ARCHITECTURE.md` to answer: how do the layers fit together?
- Use `DOMAIN.md` to answer: what does this business concept mean?
- Use `INVARIANTS.md` to answer: what must not be broken?
- Use `WORKFLOWS.md` to answer: how should this kind of change be made?
- Use `CHANGE_PROTOCOL.md` to answer: what must be checked and documented after a change?

## Task Shortcuts

For endpoint or route changes, read:

- `PROJECT_MAP.md`
- `ARCHITECTURE.md`
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
- `WORKFLOWS.md`
- `CHANGE_PROTOCOL.md`

For Solana transaction changes, read:

- `ARCHITECTURE.md`
- `DOMAIN.md`
- `INVARIANTS.md`
- `WORKFLOWS.md`

For auth, JWT, wallet, file upload, proxy, or secret-handling changes, read:

- `ARCHITECTURE.md`
- `INVARIANTS.md`
- `CHANGE_PROTOCOL.md`

## Documentation Rules

- Write for agents, not end users.
- Prefer exact paths and module names.
- Do not include secrets or real local environment values.
- Do not duplicate large code snippets.
- Update these files when routes, architecture, domain behavior, invariants, workflows, or required checks change.

