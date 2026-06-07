# Auth

The backend uses wallet-related authentication and JWT behavior.

## Main Files

- `src/server/public_server.rs`
- `src/server/auth_validation.rs`
- `src/middleware/jwt.rs`
- `src/services/auth_service.rs`
- `src/services/wallet_service.rs`
- `src/services/jwt_service.rs`

## Flow Overview

1. Client starts an auth session.
2. Client confirms login with wallet/signature data.
3. Backend validates the wallet signature.
4. Backend issues or validates JWT-related auth state.
5. Protected routes use middleware/extractors to identify the user.

## DTOs

Auth DTOs live in `src/api/dto.rs`.

- `StartSessionResponseDto`: `nonce`, `jwt`.
- `ConfirmLoginRequestDto`: `wallet_address`, `signature`, `jwt`.
- `ConfirmLoginResponseDto`: `jwt`, `is_new_user`.
- `WalletInfoResponseDto`: `creation_time`, `balance`, `tx_amount`.
- `PremarketInfoResponseDto`: `users`, `end_timestamp`, `goal_sol`, `max_sol`, `mint`, `name`, `symbol`, `uri`, `creator`.

## Security Notes

- Do not bypass wallet signature validation.
- Do not weaken JWT validation on protected behavior.
- Mutating endpoints should validate user permissions and ownership.
- Do not expose secrets or raw internal errors in API responses.
