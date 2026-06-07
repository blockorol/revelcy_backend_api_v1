# Solana Integration

The backend builds and supports Solana-related transactions for premarket and vesting flows.

## Main Files

- `src/services/solana_service.rs`: older/general Solana helpers.
- `src/services/solana_service_v2`: current transaction builder modules.
- `src/models/premarket.rs`: transaction params, built transaction models, on-chain data models.
- `src/services/premarket_service.rs`: premarket orchestration.
- `src/services/vesting_service.rs`: vesting orchestration.

## Transaction Builders

- Create premarket.
- Join premarket.
- Out/leave premarket.
- Finish premarket.
- Kill/cancel premarket.
- Extend premarket.
- Update URI.
- Update premarket data.
- Claim tokens.
- Withdraw vesting.

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
