# Workflows

Use these recipes when making code changes. Keep changes scoped and update agent docs when the structure, routes, schema, domain behavior, invariants, or workflows change.

## Add Or Change An Endpoint

1. Find the relevant scope in `src/server/mod.rs`.
2. Edit the matching server file in `src/server/`.
3. Add or update request/response DTOs in `src/api/` if needed.
4. Put business logic in the matching `src/services/*_service.rs`.
5. Put SQL in the matching `src/storage/*_repo.rs`.
6. Update `docs/agents/PROJECT_MAP.md` if route names, scope ownership, or module ownership changed.
7. Run `cargo fmt --check`, `cargo check`, and tests that cover the path.

## Add A Premarket Feature

1. Start with `src/server/premarket_server.rs` to find the current handler pattern.
2. Check validation in `src/server/premarket_validation.rs`.
3. Implement business behavior in `src/services/premarket_service.rs`.
4. Update persistence in `src/storage/premarket_repo.rs`.
5. Update domain models in `src/models/premarket.rs`.
6. Update API models in `src/api/premarket.rs` if external shapes change.
7. If the feature touches transactions, update the relevant module under `src/services/solana_service_v2/`.
8. Update `docs/agents/DOMAIN.md` and `docs/agents/INVARIANTS.md` if behavior changed.

## Add A Database Field

1. Add a new numbered migration in `migrations/`.
2. Update `src/storage/models.rs`.
3. Update affected repository queries.
4. Update service/domain/API models as needed.
5. Check list/get endpoints for default behavior and backward compatibility.
6. Update `docs/agents/PROJECT_MAP.md` if schema ownership or migration themes changed.
7. Run database-backed checks if available.

## Change Auth Or JWT Behavior

1. Inspect `src/server/public_server.rs`, `src/server/auth_validation.rs`, and `src/middleware/jwt.rs`.
2. Inspect `src/services/auth_service.rs`, `src/services/wallet_service.rs`, and `src/services/jwt_service.rs`.
3. Keep wallet signature validation and JWT validation aligned.
4. Avoid changing public error details unless needed.
5. Update `docs/agents/INVARIANTS.md` if the security model changes.
6. Run focused tests or add them if the change is security-sensitive.

## Change Solana Transaction Logic

1. Identify the matching transaction module in `src/services/solana_service_v2/`.
2. Check shared helpers in `env.rs`, `constants.rs`, `contract_specific.rs`, `solana_methods.rs`, and `utils.rs`.
3. Check input/output models in `src/models/premarket.rs`.
4. Check any service calls in `src/services/premarket_service.rs` or `src/services/vesting_service.rs`.
5. Preserve network selection and signer behavior.
6. Update `docs/agents/DOMAIN.md` and `docs/agents/INVARIANTS.md` if transaction semantics changed.

## Change Whitelist Behavior

1. Inspect `src/server/whitelist_handlers.rs`.
2. Update `src/services/whitelist_service.rs`.
3. Update `src/storage/whitelist_repo.rs`.
4. Check `src/models/whitelist.rs` and `src/api/whitelist.rs`.
5. Update invariants if permissions, approval, rejection, or application semantics changed.

## Change Vesting Behavior

1. Inspect `src/server/vesing_server_handler.rs`.
2. Update `src/services/vesting_service.rs`.
3. Update `src/storage/vesting_repo.rs`.
4. Check `src/models/vesting.rs` and `src/api/vesting.rs`.
5. If chain withdrawal behavior changes, update `src/services/solana_service_v2/tx_withdraw_vesting.rs` and `src/services/solana_service_v2/vesting.rs`.
6. Update domain and invariants docs if schedule or withdrawal semantics changed.

## Change Agent Documentation

1. Start from `docs/agents/README.md`.
2. Keep `AGENTS.md` short and focused on universal operating rules.
3. Put locations and ownership in `PROJECT_MAP.md`.
4. Put layer/data-flow design in `ARCHITECTURE.md`.
5. Put business concepts in `DOMAIN.md`.
6. Put must-preserve constraints in `INVARIANTS.md`.
7. Put repeatable edit recipes in `WORKFLOWS.md`.
8. Put check/update requirements in `CHANGE_PROTOCOL.md`.

