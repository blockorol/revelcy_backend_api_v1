# Open Source Checklist

Use this before making the repository public.

## Required Decisions

- Choose a license.
- Decide which deployment files should remain public.
- Decide whether any generated artifacts should be removed.

## Safety Checks

- Scan git history for secrets.
- Confirm `.env` and `.env.*` are ignored.
- Confirm no private keys, JWT secrets, RPC credentials, or tokens are tracked.
- Review Docker and Railway files for sensitive values.

## Documentation

- README explains the project and quick start.
- Configuration docs list env vars without real secrets.
- Development docs explain local setup.
- Security policy exists.
- Contributing guide exists.
- API and architecture docs are present.

## Quality

- Formatting check passes.
- Cargo check passes.
- Tests pass or gaps are documented.
- Dependency risks are documented.
