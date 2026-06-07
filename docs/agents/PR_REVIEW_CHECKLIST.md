# PR Review Checklist

Use this file when the user asks for a review. Findings should lead the response, ordered by severity, with file/line references.

## Review Priorities

Check for:

- Bugs.
- Security regressions.
- API contract breaks.
- Model-level boundary violations.
- Database migration/data risks.
- Solana transaction/signing risks.
- Missing tests.
- Documentation drift that will mislead future agents.

## General Checklist

- Does the change match the user request?
- Are unrelated files changed?
- Are user changes preserved?
- Are public routes/methods/DTOs changed intentionally?
- Are API/internal/storage model levels respected?
- Are errors mapped safely?
- Are secrets or local env values introduced?
- Are tests or checks appropriate for the risk?
- Are human-facing docs updated when public/developer behavior changed?
- Are agent docs updated if routes, behavior, schema, architecture, or workflows changed?

## API Review

- Route path and method stable?
- Request/response fields stable?
- Auth requirements unchanged or intentionally changed?
- Error shape stable?
- Frontend-sensitive behavior noted?

Read:

- `API_CONTRACTS.md`
- `FRONTEND_BACKEND_CONTRACT.md`
- `ERROR_HANDLING.md`

## Database Review

- New migration added instead of old migration edited?
- SQLx row models match schema?
- Repository queries match schema and expected nullability?
- Delete/cascade semantics safe?
- API exposure of DB fields intentional?

Read:

- `DATABASE.md`
- `MAPPING_GUIDE.md`

## Solana Review

- Network selection explicit?
- Program constants intended?
- Account derivation correct?
- Signer assumptions clear?
- Instruction order intentional?
- Transaction response still matches frontend/API expectations?

Read:

- `SOLANA.md`
- `RISK_REGISTER.md`

## Auth And Security Review

- JWT validation preserved?
- Wallet signature flow preserved?
- Mutating endpoints validate permission/ownership?
- File/proxy paths safe?
- No sensitive logging or public leakage?

Read:

- `AUTH_SECURITY.md`
- `INVARIANTS.md`

## Final Review Response Shape

For reviews, lead with findings:

- Severity.
- File/line reference.
- Problem.
- Impact.
- Suggested fix.

Then include open questions, test gaps, and a short summary only after findings.
