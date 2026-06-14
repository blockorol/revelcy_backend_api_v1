# Revelcy Backend API

Rust backend API for Revelcy. The service uses Actix Web, SQLx/Postgres, JWT authentication, Solana wallet signature validation, file/IPFS helpers, and Solana transaction builders for premarket and vesting flows.

## Stack

- Rust 2021
- Actix Web
- SQLx + Postgres
- JWT
- Solana SDK/client
- Docker/Railway deployment files

## Quick Start

```bash
cargo run --bin revelcy-backend-api
```

Required local environment includes at least:

- `DATABASE_URL`
- `SOLANA_RPC`

See [Configuration](docs/CONFIGURATION.md) for the full environment list.

## Documentation

- [Developer setup](docs/DEVELOPMENT.md)
- [Configuration](docs/CONFIGURATION.md)
- [API overview](docs/API.md)
- [API examples](docs/API_EXAMPLES.md)
- [Architecture](docs/ARCHITECTURE.md)
- [Database](docs/DATABASE.md)
- [Local database](docs/LOCAL_DATABASE.md)
- [Solana integration](docs/SOLANA.md)
- [Auth and wallet login](docs/AUTH.md)
- [Testing](docs/TESTING.md)
- [Deployment](docs/DEPLOYMENT.md)
- [Release process](docs/RELEASE_PROCESS.md)
- [Dependencies](docs/DEPENDENCIES.md)
- [Troubleshooting](docs/TROUBLESHOOTING.md)

Agent-facing documentation lives in [AGENTS.md](AGENTS.md) and [docs/agents](docs/agents/README.md).

## Notes

The current project notes a dependency constraint: `sqlx` version `0.7` has a conflict with `solana-sdk 1.17`. Check [Dependencies](docs/agents/DEPENDENCIES.md) before upgrading SQLx or Solana-related crates.
