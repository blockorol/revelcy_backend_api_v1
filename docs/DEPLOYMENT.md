# Deployment

The repository includes Docker and Railway deployment files.

## Files

- `Dockerfile`: main backend image.
- `entrypoint.sh`: runtime entrypoint.
- `docker-compose.yml`: compose setup.
- `railway.toml`: Railway config.
- `Dockerfile.pump_gen`, `docker-compose.pupm_gen.yml`, `railway_key_gen.toml`: key-generator related deployment files.

## Main Docker Shape

The main Dockerfile:

1. Builds a Rust/musl binary.
2. Builds static OpenSSL.
3. Installs Goose migration tool.
4. Uses an Alpine runtime image.
5. Copies migrations and `entrypoint.sh`.

`entrypoint.sh` waits for Postgres with `pg_isready`, then applies migrations with Goose:

```sh
goose -dir /migrations postgres "$DATABASE_URL" up
```

## Deployment Notes

- Ensure all required environment variables are configured in the deployment environment.
- Do not bake secrets into images.
- Review migration startup behavior before changing `entrypoint.sh`.
