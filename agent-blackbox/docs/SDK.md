# Upmarto SDK & Product Layer

Developer tools for capturing AI agent activity into Upmarto.

```
IDE (Cursor) → @upmarto/cursor → @upmarto/sdk → Local Queue → POST /event → Backend
CLI (upmarto) → upmarto-sdk (Rust) → POST /event /explain
```

## Packages

| Package | Path | Description |
|---------|------|-------------|
| Rust SDK | `upmarto-sdk-rust/` | Async tokio client, queue, retry |
| TypeScript SDK | `upmarto-sdk-ts/` | `@upmarto/sdk` — Node + browser |
| CLI | `upmarto-cli/` | `upmarto init`, `track`, `workflow`, `session`, `explain` |
| Cursor | `upmarto-cursor/` | Auto-capture hooks via SDK |

## Quick start (< 2 minutes)

### 1. Start backend

```bash
cd agent-blackbox
cargo run
# Note the API base URL from startup log
```

### 2. Initialize project

```bash
cargo run -p upmarto-cli -- init --api-url http://127.0.0.1:PORT
```

Or manually copy `.upmarto/config.json.example` → `.upmarto/config.json`.

### 3. Build SDK + Cursor hooks

```bash
cd upmarto-sdk-ts && npm install && npm run build
cd ../upmarto-cursor && npm install && npm run build
```

### 4. Install Cursor hooks

```bash
cp upmarto-cursor/hooks.json .cursor/hooks.json
```

Restart Cursor. File edits, commands, and tests are captured automatically.

### 5. Verify

```bash
cargo run -p upmarto-cli -- workflow   # isolated 6-event bug-fix scenario
cargo run -p upmarto-cli -- explain    # uses active session — no manual ID
```

Open the UI timeline to see captured events.

---

## Configuration

### Project: `.upmarto/config.json`

```json
{
  "api_url": "http://127.0.0.1:50521",
  "project_id": "auto",
  "auto_capture": true,
  "batch_size": 50,
  "flush_interval_ms": 2000,
  "retry_max": 5
}
```

### Global: `~/.upmarto/config.json`

Same schema. Project config overrides global.

### Environment

- `UPMARTO_URL` — overrides `api_url`

---

## Rust SDK

```rust
use upmarto_sdk::{EventType, TrackEvent, Upmarto};
use serde_json::json;

let client = Upmarto::from_workspace(".")?;
client.session("my-session").await;
client.track(TrackEvent {
    event_type: EventType::FileModified,
    payload: json!({ "path": "src/main.rs" }),
    timestamp: None,
}).await?;
client.flush().await?;
```

## TypeScript SDK

```typescript
import { Upmarto } from "@upmarto/sdk";

const upmarto = Upmarto.fromWorkspace();
upmarto.track({
  event_type: "file_modified",
  payload: { path: "src/main.rs" },
});
await upmarto.flush();
```

## CLI

```bash
upmarto init --api-url http://127.0.0.1:50521
upmarto workflow              # isolated session + demo events
upmarto explain               # active session (or pass session_id)
upmarto track --type file_modified --payload '{"path":"a.rs"}'
upmarto flush
```

## Event pipeline

- Batch size: **50** (configurable)
- Flush interval: **2 seconds**
- Retry: exponential backoff, max 5 attempts
- Offline: events persisted to `.upmarto/queue.jsonl`, replayed on reconnect

## v1 contract

All SDKs use only frozen v1 endpoints:
- `POST /event`
- `GET /timeline`
- `POST /explain`
- `GET /config`

Backend storage and API contract are never modified by the SDK layer.
