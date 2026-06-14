# Solana

This file is for agents changing Solana transaction builders, network selection, or chain-facing data flow.

## High-Risk Rule

Treat this area as security and funds sensitive. Do not change account derivation, signer assumptions, program constants, instruction data, network selection, or transaction ordering casually.

## Files

- `src/services/solana_service.rs`: older/general Solana helpers.
- `src/services/solana_rpc_client.rs`: central nonblocking RPC client factory, timeout, and network URL selection.
- `src/services/solana_service_v2/`: current transaction-building modules.
- `src/services/solana_service_v2/env.rs`: network and env resolution.
- `src/services/solana_service_v2/constants.rs`: chain/program constants.
- `src/services/solana_service_v2/contract_specific.rs`: program-specific helpers.
- `src/services/solana_service_v2/solana_methods.rs`: generic Solana helper methods.
- `src/services/solana_service_v2/utils.rs`: shared transaction utilities.
- `src/models/premarket.rs`: transaction input/output and on-chain data models.
- `src/services/premarket_service.rs`: premarket domain orchestration around transactions.
- `src/services/vesting_service.rs`: vesting domain orchestration around transactions.

## Transaction Modules

- `tx_create_premarket.rs`: create premarket transaction.
- `tx_join_premarket.rs`: join transaction.
- `tx_out_premarket.rs`: out/leave transaction.
- `tx_finish_premarket.rs`: finish transaction.
- `tx_kill_premarket.rs`: kill/cancel transaction.
- `tx_extend_premarket.rs`: extend deadline transaction.
- `tx_update_uri.rs`: update metadata URI transaction.
- `tx_update_premarket_data.rs`: update premarket data transaction.
- `tx_claim_tokens.rs`: claim tokens transaction.
- `tx_withdraw_vesting.rs`: vesting withdrawal transaction.
- `vesting.rs`: vesting-specific Solana helpers.

## Environment

Solana-related env names live under `src/config/`; Solana modules should use config accessors instead of reading env vars directly.

Create nonblocking RPC clients through `src/services/solana_rpc_client.rs` or the `solana_service_v2::make_async_rpc_client` wrapper. Do not call `RpcClient::new`, `new_with_timeout`, or duplicate RPC timeout selection in transaction modules.

Important names:

- `SOLANA_RPC`
- `SOLANA_DEVNET_RPC`
- `SOLANA_MAINNET_RPC`
- `NETWORK`
- `REVELCY_AUTH_PRIVATE_KEY`
- `REVELCY_AUTH_PRIVATE_KEY_DEV`
- `REVELCY_AUTH_PRIVATE_KEY_MAIN`
- `PURPLE_PROGRAM_ID_DEV`
- `PURPLE_PROGRAM_ID_MAIN`

`REVELCY_AUTH_PRIVATE_KEY_DEV` and `REVELCY_AUTH_PRIVATE_KEY_MAIN` are required by API startup validation because transaction builders need network-specific signer material. `PURPLE_PROGRAM_ID_DEV` and `PURPLE_PROGRAM_ID_MAIN` have code fallbacks but should be reviewed carefully before relying on them.

Do not log private key material, signatures, seed material, or secret config.

## Interface Level Rule

Solana builders should use internal/domain models from `src/models`, not API DTOs, storage rows, or database pools directly. Resolve database-backed inputs such as mint key material in `src/services/*_service.rs` before calling `src/services/solana_service_v2/*` builders.

Preferred flow:

1. Handler receives API DTO.
2. Service validates and resolves database/domain state.
3. Service calls Solana builder with internal/domain params.
4. Builder returns internal transaction output.
5. Handler maps output to API response.

## Review Checklist

For Solana changes, verify:

- Network selection is explicit.
- Program constants are unchanged unless intended.
- Account derivation is correct.
- Signer expectations are clear.
- Instruction order is intentional.
- Transaction output shape still matches API contract.
- Related premarket/vesting state updates remain consistent.

