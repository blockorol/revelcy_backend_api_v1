# Solana Integration

The backend builds and supports Solana-related transactions for premarket and vesting flows.

## Main Files

- `src/services/solana_service.rs`: older/general Solana helpers.
- `src/services/solana_service_v2`: current transaction builder modules.
- `src/models/premarket.rs`: transaction params, built transaction models, on-chain data models.
- `src/services/premarket_service.rs`: premarket orchestration.
- `src/services/vesting_service.rs`: vesting orchestration.

## Transaction Builders

- `tx_create_premarket.rs`: create premarket.
- `tx_join_premarket.rs`: join premarket.
- `tx_out_premarket.rs`: out/leave premarket.
- `tx_finish_premarket.rs`: finish premarket.
- `tx_kill_premarket.rs`: kill/cancel premarket.
- `tx_extend_premarket.rs`: extend premarket.
- `tx_update_uri.rs`: update URI.
- `tx_update_premarket_data.rs`: update premarket data.
- `tx_claim_tokens.rs`: claim tokens.
- `tx_withdraw_vesting.rs`: withdraw vesting.
- `vesting.rs`: vesting-specific helpers.

## API Transaction DTOs

Transaction request/response DTOs live in `src/api/premarket.rs`. Current common fields include `network`, `user_pubkey`, premarket account/pubkey identifiers, token mint identifiers, and base64 unsigned transaction data for sign/send flows.

`TxOnlyResponse` returns `transaction`. `CreatePremarketTxResponse` returns `transaction`, `premarket_account_pda`, and `mint_address`.

## Configuration

Important variables:

- `SOLANA_RPC`
- `SOLANA_DEVNET_RPC`
- `SOLANA_MAINNET_RPC`
- `NETWORK`
- `REVELCY_AUTH_PRIVATE_KEY`

## Safety Notes

- Do not log private keys, seed material, signatures, or secret config.
- Treat account derivation, signer assumptions, program constants, and instruction order as high-risk.
- Transaction response shape is frontend-sensitive.
