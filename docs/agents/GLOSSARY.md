# Glossary

This glossary is for agents. Keep entries short and operational.

## Domain Terms

- Premarket: main launch/trading-related domain object managed through `/premarket` routes, service logic, database rows, and Solana transactions.
- Concept: premarket draft/concept data before or around full premarket creation; includes visibility and created timestamp behavior.
- Holder: user-related entry/position in a premarket; appears in join, out, claim, and holder-entry flows.
- Claim: token claim behavior after the relevant premarket/chain conditions are met.
- Vesting: schedule/withdrawal-related token behavior, with both database state and Solana withdrawal transaction support.
- Whitelist: allowlist/application/approval/rejection behavior tied to premarket participation.
- Community: token/community metadata around a premarket.
- Availability: premarket availability fields and rules exposed through update routes.
- Finish: premarket lifecycle action that finalizes a premarket.
- Kill: premarket lifecycle action that cancels/kills a premarket.
- Out: premarket lifecycle action where a holder leaves/exits a premarket.
- Extend: premarket lifecycle action that extends a deadline.
- Bonding position: token/market position data used in dynamic premarket information.
- Short link: shortened public-facing link field for premarket/community data.
- Hidden/visible: visibility behavior used by list/get endpoints and concept/premarket fields.

## Model Level Terms

- API model: external HTTP request/response DTO in `src/api`.
- Internal model: in-process service/domain model in `src/models`.
- Storage model: SQLx row/query shape in `src/storage`.

## Infrastructure Terms

- RPC: Solana RPC endpoint used to read or submit chain-related data.
- Pyth: price data integration referenced by Pyth config and response models.
- IPFS: metadata/file upload target used through IPFS/proxy helpers.

