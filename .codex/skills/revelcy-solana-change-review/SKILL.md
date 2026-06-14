---
name: revelcy-solana-change-review
description: Review Revelcy Solana transaction, account derivation, network selection, signer, program constant, instruction data, claim, vesting, and premarket chain-flow changes. Use when Codex changes or reviews src/services/solana_service.rs, src/services/solana_service_v2, Solana-related models, or transaction endpoints.
---

# Revelcy Solana Change Review

Use this skill for chain-sensitive changes.

## Workflow

1. Read `docs/agents/SOLANA.md`.
2. Read `docs/agents/RISK_REGISTER.md`.
3. Read `docs/agents/INVARIANTS.md`.
4. Inspect changed Solana modules and related models/services.

## Review Questions

- Is network selection explicit and unchanged unless intended?
- Are program constants and account derivation correct?
- Are signer assumptions clear?
- Is instruction order intentional?
- Does transaction output still match API/frontend expectations?
- Are premarket/vesting DB state transitions still consistent with chain behavior?
- Are secrets, signatures, or key material kept out of logs and responses?

Update `SOLANA.md`, `FRONTEND_BACKEND_CONTRACT.md`, and `INVARIANTS.md` when transaction semantics change.

