# Domain

This file explains domain concepts for agents. It is not user-facing product documentation.

## User

Users can have additional profile data, invite codes, username/avatar fields, and wallet-linked auth behavior.

Main areas:

- `src/server/user_server.rs`
- `src/services/user_service.rs`
- `src/services/user_info_service.rs`
- `src/storage/user_repo.rs`
- `src/storage/user_info_repo.rs`
- `src/models/user.rs`

## Authentication

Auth starts through public `/auth` routes. The flow uses session/login confirmation, wallet signature logic, and JWT handling.

Main areas:

- `src/server/public_server.rs`
- `src/server/auth_validation.rs`
- `src/services/auth_service.rs`
- `src/services/wallet_service.rs`
- `src/services/jwt_service.rs`
- `src/middleware/jwt.rs`

Keep auth validation, JWT claims, and wallet signature verification aligned.

## Premarket

Premarket is the largest domain area. It includes:

- Concept creation and visibility.
- Community/token info.
- Availability updates.
- User holder entries.
- Dynamic chain/price information.
- Lifecycle state changes.
- Solana transaction creation for create/join/out/finish/kill/extend/update/claim.

Main areas:

- HTTP handlers: `src/server/premarket_server.rs`
- Validation: `src/server/premarket_validation.rs`
- Service logic: `src/services/premarket_service.rs`
- Persistence: `src/storage/premarket_repo.rs`
- Domain models: `src/models/premarket.rs`
- API models: `src/api/premarket.rs`
- Solana transaction builders: `src/services/solana_service_v2/`

## Holder

Holder data connects a user to a premarket. Holder behavior appears in join/out/claim flows and dynamic holder data.

Main areas:

- `src/services/premarket_service.rs`
- `src/storage/premarket_repo.rs`
- `src/models/premarket.rs`

## Whitelist

Whitelist behavior is routed under `/premarket/whitelist/*` and has separate service and repository modules.

Main areas:

- `src/server/whitelist_handlers.rs`
- `src/services/whitelist_service.rs`
- `src/storage/whitelist_repo.rs`
- `src/models/whitelist.rs`
- `src/api/whitelist.rs`

## Vesting

Vesting includes API behavior, storage tables, service logic, and Solana withdrawal transaction support.

Main areas:

- `src/server/vesing_server_handler.rs`
- `src/services/vesting_service.rs`
- `src/storage/vesting_repo.rs`
- `src/models/vesting.rs`
- `src/api/vesting.rs`
- `src/services/solana_service_v2/tx_withdraw_vesting.rs`
- `src/services/solana_service_v2/vesting.rs`

## Files And IPFS

File routes serve upload/image/avatar/community image flows. IPFS and proxy logic are separate integrations.

Main areas:

- `src/server/file_server.rs`
- `src/services/file_service.rs`
- `src/services/ipfs_service.rs`
- `src/server/proxy.rs`

## Premarket Transaction Flow

1. Request hits `/premarket/tx/*`.
2. Handler validates request and auth/user context.
3. Service gathers database state and domain data.
4. Solana builder in `src/services/solana_service_v2/` constructs transaction data.
5. Handler returns built transaction or sends/signs where the endpoint is designed to do that.

## Database Change Flow

1. Migration changes schema.
2. Storage row model changes.
3. Repository queries change.
4. Service/domain/API types change as needed.
5. Agent map and invariants are updated if behavior or shape changed.

