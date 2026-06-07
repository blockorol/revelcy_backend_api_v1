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
- The API/internal/storage model boundary changes.
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

Update `docs/agents/API_CONTRACTS.md` when:

- A public route, method, request DTO, response DTO, auth requirement, or error shape changes.
- API DTO ownership changes.
- A breaking contract change is introduced intentionally.

Update `docs/agents/DATABASE.md` when:

- Migrations, repository queries, storage row models, delete semantics, or database relationships change.
- A storage model starts or stops mapping to a table/query shape.

Update `docs/agents/SOLANA.md` when:

- Transaction builders, network selection, account derivation, signer behavior, or Solana env requirements change.
- A premarket/vesting chain flow changes.

Update `docs/agents/AUTH_SECURITY.md` when:

- Auth, JWT, wallet signature validation, authorization checks, secret handling, file upload, or proxy behavior changes.

Update `docs/agents/TESTING.md` when:

- Required verification commands change.
- New test categories, fixtures, mocks, or external-service assumptions are added.

Update `docs/agents/GLOSSARY.md` when:

- A domain/codebase term is added, renamed, or clarified.
- Existing terminology is found to be ambiguous.

Update `docs/agents/RISK_REGISTER.md` when:

- A new high-risk file, flow, or failure mode is discovered.
- A risky area becomes safer or changes ownership.

Update `docs/agents/LEGACY_NOTES.md` when:

- A known imperfection, legacy module, typo, or transitional pattern is discovered or resolved.
- The project intentionally keeps a non-ideal structure for compatibility.

Update `docs/agents/ERROR_HANDLING.md` when:

- API error shape, service error handling, storage error mapping, or logging expectations change.

Update `docs/agents/DEPENDENCIES.md` when:

- Dependency versions, upgrade constraints, or sensitive dependency notes change.

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
- Update public contract details in `docs/agents/API_CONTRACTS.md`.
- Note breaking request/response changes in the final response.

### `revelcy-update-db-docs`

Trigger after migration or repository changes.

Tasks:

- Inspect new migration files.
- Inspect `src/storage/models.rs` and affected repositories.
- Update schema-related notes in `docs/agents/DATABASE.md` and `docs/agents/PROJECT_MAP.md`.

### `revelcy-update-solana-docs`

Trigger after Solana transaction, network, account, signer, or chain-flow changes.

Tasks:

- Inspect changed `src/services/solana_service_v2` files.
- Inspect related models in `src/models/premarket.rs`.
- Update `docs/agents/SOLANA.md`.

### `revelcy-update-testing-docs`

Trigger after test command, fixture, mock, or verification strategy changes.

Tasks:

- Inspect changed tests and config.
- Update `docs/agents/TESTING.md`.

### `revelcy-security-pass`

Trigger after auth, JWT, wallet, Solana, file upload, proxy, or secret-handling changes.

Tasks:

- Review changed files against `docs/agents/INVARIANTS.md`.
- Check for secret logging or tracked secret values.
- Check authorization paths for mutations.
- Report residual risks.

### `revelcy-update-risk-docs`

Trigger after discovering risky files, legacy behavior, or dependency constraints.

Tasks:

- Update `docs/agents/RISK_REGISTER.md`.
- Update `docs/agents/LEGACY_NOTES.md` or `docs/agents/DEPENDENCIES.md` when relevant.

### `revelcy-update-error-docs`

Trigger after API/service/storage error behavior changes.

Tasks:

- Inspect error mapping in `src/api/errors.rs` and affected handlers/services.
- Update `docs/agents/ERROR_HANDLING.md`.
