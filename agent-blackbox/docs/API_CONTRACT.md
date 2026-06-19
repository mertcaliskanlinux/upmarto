# Upmarto — API Contract (v1)

**Status: FROZEN** — Engine development is complete. Breaking changes require version bump.

## Product

**Upmarto** — *Memory and Reasoning for AI Agents* — is a local-first AI event intelligence system that records coding agent behavior and explains **why** actions happened.

| Layer | Endpoint | Purpose |
|-------|----------|---------|
| Runtime | `GET /config` | API base URL discovery |
| Event | `POST /event` | Immutable append-only capture |
| Timeline | `GET /timeline` | Deterministic session replay |
| Session | `GET /session/:id`, `GET /project/:id/sessions` | Session metadata |
| Reasoning | `POST /explain` | Causal explanation (core feature) |

## Versioning strategy

| Constant | Value | Bump when |
|----------|-------|-----------|
| `API_VERSION` | `v1` | Any breaking HTTP contract change |
| `EVENT_SCHEMA_VERSION` | `v1` | New/removed event types or ingest schema change |
| `EXPLAIN_SCHEMA_VERSION` | `v1` | Any change to `/explain` response fields |

All responses include `api_version`. Explain responses additionally include `explain_schema_version`.

Future breaking changes ship as `v2` types alongside `v1` — never mutate v1 in place.

## Event types (frozen)

```
file_opened | file_modified | file_created | command_executed
test_run | test_failed | test_passed | git_commit | agent_message
```

## GET /config

Runtime discovery — returns the client-facing API base URL (no fixed port assumed).

**Response** `200 OK`
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

When `PUBLIC_BASE_URL` is set on the server, `api_base_url` reflects that value (reverse proxy / ingress).

## POST /event

**Request**
```json
{
  "project_id": "string (required)",
  "session_id": "string (required)",
  "event_type": "file_modified",
  "timestamp": 1700000000001,
  "payload": {}
}
```

**Validation**
- `project_id`, `session_id` — non-empty
- `payload` — must be JSON object
- `timestamp` — optional, non-negative milliseconds

**Response** `201 Created`
```json
{
  "api_version": "v1",
  "event_schema_version": "v1",
  "event": { "id": "uuid", "timestamp": 0, "project_id": "", "session_id": "", "event_type": "", "payload": {} }
}
```

## GET /timeline?session_id={id}

Returns events in **strict timestamp order** for deterministic replay.

**Response** `200 OK`
```json
{
  "api_version": "v1",
  "session_id": "",
  "events": [],
  "summary": { "total_events": 0, "file_events": 0, "command_events": 0, "test_events": 0, "git_events": 0, "agent_messages": 0 }
}
```

## GET /session/:id

**Response** `200 OK`
```json
{
  "api_version": "v1",
  "session": { "id": "", "project_id": "", "started_at": 0, "ended_at": null, "event_count": 0 }
}
```

## GET /project/:id/sessions

**Response** `200 OK`
```json
{
  "api_version": "v1",
  "project_id": "",
  "sessions": []
}
```

## POST /explain

**Request**
```json
{
  "session_id": "string (required)",
  "event_id": "optional-uuid"
}
```

When `event_id` is provided, reasoning is scoped to events up to and including that event.

**Response** `200 OK` — **FROZEN SCHEMA v1**
```json
{
  "api_version": "v1",
  "explain_schema_version": "v1",
  "summary": "human-readable explanation",
  "root_cause": "main reason for actions",
  "decision_chain": ["event → interpretation → reason"],
  "problem_statement": "what the agent was trying to solve",
  "resolution_flow": "how it was solved step by step"
}
```

### Explain field semantics

| Field | Meaning |
|-------|---------|
| `summary` | Executive overview for a developer |
| `root_cause` | Primary trigger (test failure, directive, file change) |
| `decision_chain` | Ordered causal links per event/pattern |
| `problem_statement` | The problem the agent was solving |
| `resolution_flow` | Step-by-step resolution narrative |

Explain output is **deterministic** — same event sequence always produces the same explanation. No LLM, no external data.

## Errors

**Response** `4xx/5xx`
```json
{
  "api_version": "v1",
  "error": "description"
}
```

| Code | When |
|------|------|
| `400` | Validation failure |
| `404` | Session, project, or event not found |
| `500` | Internal storage error |

## Product surfaces (UX model)

| Surface | Role |
|---------|------|
| **Session** | Unit of analysis — all views scoped to a session |
| **Timeline** | Replay interface — ordered action stream |
| **Explain** | Reasoning interface — answers "why?" |

## Hard constraints

- No new event types without `EVENT_SCHEMA_VERSION` bump
- No `/explain` schema changes without `EXPLAIN_SCHEMA_VERSION` bump
- JSONL + SQLite storage (no redesign)
- No LLM dependency
- No external services
- Local-first only
