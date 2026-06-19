# Upmarto

[![CI](https://github.com/mertcaliskanlinux/upmarto/actions/workflows/ci.yml/badge.svg)](https://github.com/mertcaliskanlinux/upmarto/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://github.com/mertcaliskanlinux/upmarto/blob/main/agent-blackbox/LICENSE)
[![GitHub](https://img.shields.io/badge/github-mertcaliskanlinux%2Fupmarto-181717?logo=github)](https://github.com/mertcaliskanlinux/upmarto)

**Memory and Reasoning for AI Agents**

Understand What Your AI Agents Did — And Why.

Upmarto records every developer and agent action, replays full session timelines, and explains **why** actions happened — deterministically, without external services.

> This is not a logging system, telemetry pipeline, or generic event store.  
> It is an **AI agent behavior debugger** with causal reasoning.

## What is Upmarto?

Upmarto answers three questions for every coding session:

1. **What happened?** — append-only event capture (`POST /event`)
2. **What was the sequence?** — ordered session replay (`GET /timeline`)
3. **Why did it happen?** — deterministic causal explanation (`POST /explain`)

### Core value

| Capability | API | Product surface |
|------------|-----|-----------------|
| Capture every action | `POST /event` | Event ingest |
| Replay session | `GET /timeline` | Timeline view |
| Explain causality | `POST /explain` | Reasoning view |
| Session metadata | `GET /session/:id` | Session view |

**Session** is the unit of analysis. All timeline replay and explanations are scoped to a session.

## Quick start

```bash
cd upmarto   # or agent-blackbox/ until the repo folder is renamed
cargo run
# Note the printed "API base URL" — port is OS-assigned when APP_PORT=0
```

See [docs/DEPLOYMENT.md](docs/DEPLOYMENT.md) for production, Docker, and env configuration.

### Passive capture (IDE integration)

Install once — events flow automatically to `POST /event`:

```bash
# 1. Build the TypeScript SDK and Cursor hooks
cd upmarto-sdk-ts && npm install && npm run build
cd ../upmarto-cursor && npm install && npm run build

# 2. Initialize project config (API URL from cargo run startup log)
cargo run -p upmarto-cli -- init --api-url http://127.0.0.1:PORT

# 3. Install Cursor hooks
mkdir -p .cursor
cp upmarto-cursor/hooks.json .cursor/hooks.json
# Restart Cursor after saving hooks.json
```

Full guide: [docs/SDK.md](docs/SDK.md)

```env
APP_HOST=127.0.0.1
APP_PORT=0
DATABASE_PATH=./data/events.log
SQLITE_PATH=./data/metadata.db
TEST_MODE=true   # optional — uses ./data/test for isolated storage
```

## Architecture

```
POST /event  →  JSONL append log  +  SQLite index
GET /timeline  →  offset lookup  →  ordered replay
POST /explain  →  WHY engine  →  frozen v1 reasoning schema
```

Storage: `data/events.log` (JSONL) + `data/metadata.db` (SQLite index).

## API contract (v1 — frozen)

Full contract: [docs/API_CONTRACT.md](docs/API_CONTRACT.md)

### Event layer — `POST /event`

```bash
# API_BASE from startup log or: curl $API_BASE/config
curl -X POST "$API_BASE/event" \
  -H "Content-Type: application/json" \
  -d '{
    "project_id": "my-app",
    "session_id": "sess-001",
    "event_type": "file_modified",
    "timestamp": 1700000001001,
    "payload": { "path": "src/auth.rs" }
  }'
```

**Event types (frozen):** `file_opened`, `file_modified`, `file_created`, `command_executed`, `test_run`, `test_failed`, `test_passed`, `git_commit`, `agent_message`

### Timeline layer — `GET /timeline?session_id={id}`

Returns events in strict timestamp order for deterministic replay.

```bash
curl "$API_BASE/timeline?session_id=sess-001"
```

### Session layer

```bash
curl "$API_BASE/session/sess-001"
curl "$API_BASE/project/my-app/sessions"
```

### Reasoning layer — `POST /explain` (core feature)

```bash
curl -X POST "$API_BASE/explain" \
  -H "Content-Type: application/json" \
  -d '{ "session_id": "sess-001" }'
```

**Response schema (v1 — product contract):**

```json
{
  "api_version": "v1",
  "explain_schema_version": "v1",
  "summary": "Agent attempted to fix a failure in auth_test...",
  "root_cause": "Test failure in auth_test — token expired",
  "decision_chain": [
    "test_failed (auth_test) → regression detected → motivates investigation",
    "file_modified (auth.rs) → apply fix or implement solution"
  ],
  "problem_statement": "Resolve failing test: auth_test",
  "resolution_flow": "1. Detected failure in auth_test. 2. Modified auth.rs. 3. Validated fix."
}
```

| Field | Purpose |
|-------|---------|
| `summary` | Human-readable overview |
| `root_cause` | Primary trigger for the session |
| `decision_chain` | Causal links: event → interpretation → reason |
| `problem_statement` | What the agent was trying to solve |
| `resolution_flow` | Step-by-step how it was resolved |

Explain is **fully deterministic** — same events always produce the same output. No LLM.

Optional `event_id` scopes reasoning to a point in time:

```json
{ "session_id": "sess-001", "event_id": "550e8400-e29b-41d4-a716-446655440000" }
```

## Versioning

| Version constant | Current | Bump when |
|------------------|---------|-----------|
| `API_VERSION` | v1 | Breaking HTTP contract |
| `EVENT_SCHEMA_VERSION` | v1 | New event types |
| `EXPLAIN_SCHEMA_VERSION` | v1 | Explain response field changes |

All responses include `api_version`. Breaking changes ship as v2 — v1 is never mutated.

## Testing

```bash
cargo test              # CI-safe, isolated, no manual setup
cargo insta review      # review snapshot changes to /explain output
```

## Product boundaries

**This system IS:**
- AI agent behavior debugger
- Causal reasoning engine for developer actions
- Session replay + explanation system

**This system IS NOT:**
- A logging system
- A telemetry pipeline
- A generic event store

**Hard rules (product freeze):**
- No engine changes without version bump
- No storage redesign
- No LLM dependency
- Local-first only

## Design constraints

- Single-node, handles ~100k events
- No auth (MVP)
- No distributed storage

## Migration

Renamed from **Agent Blackbox** — see [docs/MIGRATION.md](docs/MIGRATION.md).
