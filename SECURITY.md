# Security

## Reporting

If you discover a security issue, do not publish exploit details publicly. Report it privately to the project maintainers.

## Sensitive Areas

- Wallet signature validation.
- JWT creation and validation.
- Solana transaction construction and signing assumptions.
- Private keys, seed material, and RPC credentials.
- File upload and proxy behavior.
- Database state transitions for premarket, claim, whitelist, and vesting flows.

## Secrets

Do not commit:

- `.env` or `.env.*` files.
- JWT secrets.
- Solana private keys.
- RPC credentials.
- Pyth/IPFS secret tokens.
- Seed material or signed sensitive payloads.

## Safe Logging

Logs should not include private keys, JWT secrets, seed material, raw secret config, or sensitive signed payloads.
