# API Examples

This file gives lightweight examples for API consumers. Replace placeholder values with real local/dev values.

## Auth Session

Start a wallet login session:

```bash
curl -X GET "http://localhost:8080/auth/start_session"
```

Confirm login:

```bash
curl -X POST "http://localhost:8080/auth/confirm_login" \
  -H "Content-Type: application/json" \
  -d '{ "wallet_address": "WALLET_PUBLIC_KEY", "signature": "SIGNATURE", "jwt": "SESSION_JWT" }'
```

Relevant DTOs in `src/api/dto.rs`:

- `StartSessionResponseDto`: `nonce`, `jwt`
- `ConfirmLoginRequestDto`: `wallet_address`, `signature`, `jwt`
- `ConfirmLoginResponseDto`: `jwt`, `is_new_user`

## User Search

```bash
curl -X POST "http://localhost:8080/user/search" \
  -H "Authorization: Bearer JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{ "input": "alice", "limit": 10 }'
```

Relevant DTO: `SearchUsersRequestDto` in `src/api/user.rs`.

## Premarket List

```bash
curl -X GET "http://localhost:8080/premarket/get_list"
```

Current query DTO requires `network`, `cursor`, and `limit`:

```bash
curl -X GET "http://localhost:8080/premarket/get_list?network=devnet&cursor=0&limit=20"
```

## Premarket Main Info

```bash
curl -X GET "http://localhost:8080/premarket/get_main_info?network=devnet&premarket_id=PREMARKET_ID"
```

`GetMainInfoQuery` supports `premarket_id`, `premarket_name`, and `network`.

## Build Premarket Transaction

```bash
curl -X POST "http://localhost:8080/premarket/tx/create" \
  -H "Authorization: Bearer JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{ "network": "devnet", "user_pubkey": "USER_PUBKEY", "uri": "ipfs://...", "image_url": "https://example.test/image.png", "premarket_pubkey": "PREMARKET_PUBKEY", "creator_allocate_lamp": "1000000" }'
```

Transaction request/response shapes are frontend-sensitive. Check `src/api/premarket.rs`, `src/models/premarket.rs`, and `docs/SOLANA.md` when these examples need to be made exact.

## Whitelist Apply

```bash
curl -X POST "http://localhost:8080/premarket/whitelist/apply" \
  -H "Authorization: Bearer JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{ "network": "devnet", "premarket_id": "PREMARKET_UUID" }'
```

## Update Vesting Info

```bash
curl -X POST "http://localhost:8080/premarket/vesting/update_info" \
  -H "Authorization: Bearer JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{ "user_pubkey": "USER_PUBKEY", "network": "devnet", "premarket_pubkey": "PREMARKET_PUBKEY", "vesting_period_sec": 2592000, "unlock_at_launch_percent": 10, "enabled": true }'
```

## Error Shape

Error responses are API contracts. Check `src/api/errors.rs` and handlers before documenting exact status/body examples.

## DTO Field Reference

This section mirrors the current public DTO names and fields. The source of truth remains `src/api`.

### Auth DTOs

`StartSessionResponseDto`:

- `nonce: string`
- `jwt: string`

`ConfirmLoginRequestDto`:

- `wallet_address: string`
- `signature: string`
- `jwt: string`

`ConfirmLoginResponseDto`:

- `jwt: string`
- `is_new_user: boolean`

### User DTOs

`SearchUsersRequestDto`:

- `input: string`
- `limit: integer`

`SearchUsersResponseDto`:

- `items: UserDto[]`

`GetUsersShortListRequestDto`:

- `addresses: string[]`

`UserDto`:

- `id: string`
- `username: string | null`
- `avatar_url: string | null`
- `wallets: string[]`

### Premarket Query DTOs

`GetMainInfoQuery`:

- `premarket_id?: string`
- `premarket_name?: string`
- `network: string`

`GetListQuery`:

- `network: string`
- `cursor: integer`
- `limit: integer`
- `state?: TokenState[]`
- `only_user_token?: boolean`

`GetDynamicInfoQuery`:

- `premarket_id: string`

`GetHolderEntryInfoQuery`:

- `premarket_id: string`
- `holder_wallet: string`

### Premarket Transaction DTOs

Common transaction requests use `network`, `user_pubkey`, and one or more premarket/token identifiers.

`JoinPremarketTxRequest`:

- `network: string`
- `user_pubkey: string`
- `premarket_account: string`
- `amount_sol_lamp: string`

`OutPremarketTxRequest`:

- `network: string`
- `user_pubkey: string`
- `premarket_account: string`

`FinishPremarketTxRequest` and `KillPremarketTxRequest`:

- `network: string`
- `user_pubkey: string`
- `premarket_account: string`

`ExtendPremarketTxRequest`:

- `network: string`
- `user_pubkey: string`
- `premarket_account: string`
- `new_deadline: integer`

`ClaimTokensTxRequest`:

- `network: string`
- `user_pubkey: string`
- `premarket_account: string`
- `token_mint: string`

`WithdrawVestingTxRequest`:

- `network: string`
- `user_pubkey: string`
- `token_mint: string`

`TxOnlyResponse`:

- `transaction: string`

### Whitelist DTOs

Whitelist requests use `network` and `premarket_id`.

- Add/remove/status requests accept one of `user_id` or `user_pubkey`.
- Bulk add accepts `user_ids`, `user_pubkeys`, or both.
- List requests accept `status`, `cursor`, and `limit`.

### Vesting DTOs

`UpdateVestingInfoRequest`:

- `user_pubkey: string`
- `network: string`
- `premarket_pubkey: string`
- `vesting_period_sec: integer`
- `unlock_at_launch_percent: integer`
- `enabled: boolean`

`UpdateVestingInfoResponse`:

- `ok: boolean`
