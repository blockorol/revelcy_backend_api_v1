# Change Protocol

Use this protocol after code or documentation changes. It tells agents what to verify and which agent docs to update.

## Required Behavior

- Do not change application code when the user asks only for documentation, planning, review, or analysis.
- Preserve user changes in the working tree.
- Keep changes scoped to the user request.
- Report checks that could not be run.
- Do not claim changes are verified unless the relevant command actually ran.

## Documentation Update Rules

Update `AGENTS.md` when:

- Required commands change.
- Broad repository rules change.
- Major directories or architectural layers are added/removed.
- High-risk areas change.

Update `docs/agents/README.md` when:

- A new agent document is added.
- Reading order or task shortcuts change.

Update `docs/agents/PROJECT_MAP.md` when:

- Routes are added, removed, renamed, or moved.
- Modules are added, removed, renamed, or repurposed.
- Migration themes change in a way future agents should know.
- Startup entrypoints or registered scopes change.

Update `docs/agents/ARCHITECTURE.md` when:

- Layer responsibilities change.
- Request flow changes.
- Solana integration shape changes.
- Configuration or deployment shape changes in a way agents must understand.

Update `docs/agents/DOMAIN.md` when:

- Domain concepts are added or renamed.
- Premarket, whitelist, vesting, user, auth, file, IPFS, or Solana behavior changes.
- Data flow across handler/service/storage/transaction layers changes.

Update `docs/agents/INVARIANTS.md` when:

- A rule future agents must preserve is introduced or removed.
- Auth, authorization, Solana, database state transitions, file/proxy safety, or claim/vesting semantics change.
- Existing code intentionally violates or replaces an older invariant.

Update `docs/agents/WORKFLOWS.md` when:

- A recurring change pattern appears.
- The steps for endpoints, migrations, auth, Solana, whitelist, or vesting change.
- New checks become necessary for a class of change.

## Verification Checklist

Use the narrowest checks that give confidence for the change.

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

If dependency or network-related commands fail because of the local sandbox/environment, report that clearly.

## High-Risk Review Checklist

Run an extra review pass for changes touching:

- JWT creation or validation.
- Wallet signature confirmation.
- Solana transaction construction or signing.
- Network selection and program constants.
- Database state transitions for premarket lifecycle.
- Claim, vesting, or whitelist authorization.
- File upload and proxy behavior.
- Default secret-like config values.

## Candidate Agent Skills

These are candidate Codex skills or recurring prompts for this repository.

### `revelcy-update-project-map`

Trigger after changes to routes, modules, migrations, or major ownership boundaries.

Tasks:

- Inspect `src/server/mod.rs` and route files.
- Inspect changed module declarations.
- Update `docs/agents/PROJECT_MAP.md`.

### `revelcy-update-architecture-docs`

Trigger after changes to layering, startup, configuration, deployment, or integration shape.

Tasks:

- Inspect changed entrypoints and module boundaries.
- Update `docs/agents/ARCHITECTURE.md`.

### `revelcy-update-domain-docs`

Trigger after changes to business behavior.

Tasks:

- Inspect changed service and model files.
- Update `docs/agents/DOMAIN.md`.
- Update `docs/agents/INVARIANTS.md` if behavior changed.

### `revelcy-update-api-route-docs`

Trigger after handler/API DTO changes.

Tasks:

- Inspect changed `src/server` and `src/api` files.
- Update route map in `docs/agents/PROJECT_MAP.md`.
- Note breaking request/response changes in the final response.

### `revelcy-update-db-docs`

Trigger after migration or repository changes.

Tasks:

- Inspect new migration files.
- Inspect `src/storage/models.rs` and affected repositories.
- Update schema-related notes in `docs/agents/PROJECT_MAP.md`.

### `revelcy-security-pass`

Trigger after auth, JWT, wallet, Solana, file upload, proxy, or secret-handling changes.

Tasks:

- Review changed files against `docs/agents/INVARIANTS.md`.
- Check for secret logging or tracked secret values.
- Check authorization paths for mutations.
- Report residual risks.

