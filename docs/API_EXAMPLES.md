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
  -d '{ "public_key": "WALLET_PUBLIC_KEY", "signature": "SIGNATURE", "message": "MESSAGE" }'
```

Request/response fields should be checked against `src/api/dto.rs` before using these examples as strict contracts.

## User Search

```bash
curl -X POST "http://localhost:8080/user/search" \
  -H "Authorization: Bearer JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{ "query": "alice" }'
```

## Premarket List

```bash
curl -X GET "http://localhost:8080/premarket/get_list"
```

## Premarket Main Info

```bash
curl -X GET "http://localhost:8080/premarket/get_main_info?premarket_account=PREMARKET_ACCOUNT"
```

## Build Premarket Transaction

```bash
curl -X POST "http://localhost:8080/premarket/tx/create" \
  -H "Authorization: Bearer JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{ "network": "devnet" }'
```

Transaction request/response shapes are frontend-sensitive. Check `src/api/premarket.rs`, `src/models/premarket.rs`, and `docs/SOLANA.md` when these examples need to be made exact.

## Error Shape

Error responses are API contracts. Check `src/api/errors.rs` and handlers before documenting exact status/body examples.

