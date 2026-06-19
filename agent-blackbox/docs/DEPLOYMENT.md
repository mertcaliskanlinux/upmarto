# Deployment Guide

Upmarto is environment-agnostic: all hosts, ports, and storage paths come from configuration.

## Environment variables (backend)

| Variable | Default | Description |
|----------|---------|-------------|
| `APP_HOST` | `0.0.0.0` | Bind address |
| `APP_PORT` | `0` | Listen port (`0` = OS-assigned free port) |
| `DATABASE_PATH` | `./data/events.log` | JSONL event log |
| `SQLITE_PATH` | `./data/metadata.db` | SQLite index |
| `PUBLIC_BASE_URL` | _(auto)_ | Override URL returned by `GET /config` |
| `TEST_MODE` | `false` | Isolated test storage |

Legacy aliases: `HOST`, `PORT`, `DATA_DIR`, `EVENTS_LOG_PATH`.

Copy `.env.example` to `.env` and adjust.

## Dynamic port (development)

```bash
cargo run
# [Upmarto] listening on http://127.0.0.1:54321
# API base URL: http://127.0.0.1:54321
```

Use the printed **API base URL** — never assume a fixed port.

## Frontend

| Variable | Purpose |
|----------|---------|
| `VITE_API_BASE_URL` | Direct backend URL (production builds) |
| `VITE_API_PROXY_TARGET` | Dev-only Vite proxy target |
| `VITE_DEV_PORT` | Dev server port (optional) |

**Resolution order:**

1. `VITE_API_BASE_URL` if set at build time
2. `GET /config` on same origin (runtime discovery)
3. `window.location.origin` (same-origin deploy)

### Dev workflow

```bash
# Terminal 1 — note the API base URL from startup log
cargo run

# Terminal 2 — point Vite proxy at that URL
cd ui
echo "VITE_API_PROXY_TARGET=http://127.0.0.1:ACTUAL_PORT" > .env
npm run dev
```

### Production (same-origin)

Serve the UI static build behind the same host as the API, or set `VITE_API_BASE_URL` at build time:

```bash
VITE_API_BASE_URL=https://upmarto.example.com npm run build
```

## Runtime discovery — GET /config

```bash
curl http://127.0.0.1:PORT/config
```

```json
{
  "api_version": "v1",
  "product_name": "Upmarto",
  "product_tagline": "Memory and Reasoning for AI Agents",
  "api_base_url": "http://127.0.0.1:54321",
  "host": "127.0.0.1",
  "port": 54321
}
```

Set `PUBLIC_BASE_URL` when behind a reverse proxy so clients receive the external URL.

## Integration layer

Set `UPMARTO_URL` to the backend API base URL (from startup log or `/config`):

```bash
export UPMARTO_URL=http://127.0.0.1:54321
```

No default URL is hardcoded — capture is disabled until configured.

## Docker

```bash
docker compose up --build
```

- Backend binds `APP_PORT=0` inside the container; host port mapped dynamically via Compose.
- Set `PUBLIC_BASE_URL` for external clients.
- Mount a volume for `./data` to persist events across restarts.

## Multiple instances

Each instance needs:

- Unique `DATABASE_PATH` / `SQLITE_PATH` (or separate `DATA_DIR`)
- Its own bind port (`APP_PORT=0` avoids collisions)
- Integration clients pointed at the correct `UPMARTO_URL`

Tests already use isolated temp dirs and port `0` via `Settings::for_isolated_test`.
