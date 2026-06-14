# Frontend Backend Contract

This file records backend behavior that is likely to affect the frontend. It is for agents changing backend routes, DTOs, auth, transactions, uploads, or image URLs.

## Contract Rule

Treat public route names, methods, request fields, response fields, auth behavior, and transaction response shapes as frontend-sensitive.

Do not change these casually:

- Route path.
- HTTP method.
- Required auth/session behavior.
- Required request fields.
- Response field names.
- Response nesting.
- Error status/body shape.
- Image/file URL behavior.
- Transaction response shape or encoding.

## Main Frontend-Sensitive Areas

### Auth

Routes:

- `GET /auth/start_session`
- `POST /auth/confirm_login`
- `GET /auth/wallet_info/{pubkey}`
- `GET /auth/premarket_info/{premarket_account}`

Risks:

- Wallet login flow breaks.
- JWT/session expectations change.
- Wallet info shape changes.

### User

Routes under `/user` affect profile, invite, username, avatar, user list, and search flows.

Risks:

- Profile UI fields disappear or change names.
- Search/list behavior changes.
- Avatar update/retrieval flow breaks.

### Files And Images

Routes under `/files` affect upload and image display behavior.

Risks:

- Image URLs change.
- Avatar/community image retrieval breaks.
- Upload behavior or accepted names change.

### Premarket

Routes under `/premarket` are the main frontend contract for premarket screens, forms, lists, dynamic info, whitelist, concepts, vesting updates, and transaction creation.

Risks:

- List/get pages break.
- Create/update forms break.
- Whitelist UI state breaks.
- Transaction building/signing flow breaks.

### Solana Transactions

Routes under `/premarket/tx/*` are especially frontend-sensitive.

Risks:

- Wallet signing flow no longer understands response.
- Encoded transaction data shape changes.
- Required accounts/signers change without frontend adjustment.
- Network expectations change.
- Error response shape changes can break transaction form handling. `POST /premarket/tx/kill` now uses structured `ApiError` JSON for validation/auth/build failures.

## Change Checklist

For frontend-sensitive backend changes:

- Update `API_CONTRACTS.md`.
- Update `PROJECT_MAP.md` if routes changed.
- Update `SOLANA.md` if transaction behavior changed.
- Mention frontend impact in the final response.
- Preserve backward compatibility unless the user asked for a breaking change.

