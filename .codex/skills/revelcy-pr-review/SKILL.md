---
name: revelcy-pr-review
description: Review Revelcy backend changes for bugs, regressions, security issues, API contract breaks, model boundary violations, database risks, Solana transaction risks, missing tests, and agent documentation drift. Use when the user asks Codex to review a diff, PR, branch, or recent change in this repository.
---

# Revelcy PR Review

Use this skill in review mode.

## Workflow

1. Read `docs/agents/PR_REVIEW_CHECKLIST.md`.
2. Read `docs/agents/RISK_REGISTER.md`.
3. Inspect the diff and changed files.
4. Use specialized docs as needed:
   - API: `API_CONTRACTS.md`
   - DB: `DATABASE.md`
   - Solana: `SOLANA.md`
   - Auth/security: `AUTH_SECURITY.md`
   - Mapping: `MAPPING_GUIDE.md`
5. Lead the response with findings ordered by severity.

## Finding Format

Include:

- Severity.
- File/line reference.
- Problem.
- Impact.
- Suggested fix.

If no issues are found, say that clearly and mention test gaps or residual risk.

