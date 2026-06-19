# Upmarto UI

Minimal developer UI for session replay and causal reasoning. Uses **v1 API contract only**.

## Run

Terminal 1 — backend:

```bash
cargo run
# Copy the printed API base URL (e.g. http://127.0.0.1:54321)
```

Terminal 2 — frontend:

```bash
cd ui
npm install
cp .env.example .env
# Set VITE_API_PROXY_TARGET to the backend URL from step 1
npm run dev
```

Open the URL printed by Vite (port is configurable via `VITE_DEV_PORT`).

## Configuration

Copy `.env.example` to `.env`:

| Variable | Purpose |
|----------|---------|
| `VITE_API_PROXY_TARGET` | Dev: proxy `/event`, `/timeline`, etc. to backend |
| `VITE_API_BASE_URL` | Production: direct backend URL (leave empty for same-origin + `/config`) |
| `VITE_DEFAULT_PROJECT_ID` | Default project in session list |

API base resolution: `VITE_API_BASE_URL` → `GET /config` → `window.location.origin`.

See [docs/DEPLOYMENT.md](../docs/DEPLOYMENT.md).

## Routes

| Route | Surface |
|-------|---------|
| `/sessions` | Session list (`GET /project/:id/sessions`) |
| `/timeline/:session_id` | Event replay (`GET /timeline`) |
| `/explain/:session_id` | WHY reasoning (`POST /explain`) |

## Build

```bash
npm run build
```

Output in `dist/`. For production, set `VITE_API_BASE_URL` or serve behind the same origin as the API.
