# Upmarto for VS Code

[![CI](https://github.com/mertcaliskanlinux/upmarto/actions/workflows/ci.yml/badge.svg)](https://github.com/mertcaliskanlinux/upmarto/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://github.com/mertcaliskanlinux/upmarto/blob/main/agent-blackbox/LICENSE)

**Understand what your AI agents did — and why.**

Upmarto passively captures developer and AI agent activity into a **local-first** session timeline. Replay events in order, then run a deterministic **WHY engine** that explains root causes — no cloud telemetry, no LLM guesswork.

---

## Why Upmarto?

| Problem | Upmarto answer |
|---------|----------------|
| Agent sessions are opaque | Ordered **session timeline** of every action |
| Logs don't explain causality | Rule-based **`explain`** with root cause + decision chain |
| Cloud tools add friction | **127.0.0.1** backend — your data stays on your machine |
| Offline / flaky networks | **Batch queue** with retry to `.upmarto/queue.jsonl` |

---

## Screenshots

### Session timeline

See every file edit, test, and command in sequence — scoped to a session.

![Upmarto session timeline](https://raw.githubusercontent.com/mertcaliskanlinux/upmarto/main/agent-blackbox/screenshots/01-timeline.png)

### Root-cause explain

Deterministic explanation: what failed, what changed, and how it was resolved.

![Upmarto explain output](https://raw.githubusercontent.com/mertcaliskanlinux/upmarto/main/agent-blackbox/screenshots/02-explain.png)

---

## Features

- **Passive capture** — file open/edit/save, tasks, terminal commands, extension lifecycle
- **Local batch queue** — events buffered and flushed to `POST /event` (v1 API)
- **Session-scoped analysis** — timeline and explain per session, not global noise
- **Rule-based WHY engine** — `root_cause`, `decision_chain`, `resolution_flow` without external AI
- **Works with the Upmarto CLI** — `init`, `workflow`, `explain` for onboarding and debugging

---

## Requirements

1. [Upmarto backend](https://github.com/mertcaliskanlinux/upmarto) running locally (`cargo run` in `agent-blackbox/`)
2. Project initialized: `upmarto-cli init`
3. VS Code **1.85+**

---

## Quick start

```bash
# Terminal 1 — backend
cd agent-blackbox && cargo run

# Terminal 2 — bootstrap
cargo run -p upmarto-cli -- init
cargo run -p upmarto-cli -- workflow
cargo run -p upmarto-cli -- explain
```

Install this extension, open your workspace, and events flow automatically.

---

## Configuration

| VS Code setting | Default | Description |
|-----------------|---------|-------------|
| `upmarto.enabled` | `true` | Toggle passive capture |

API URL, batch size, and retry settings come from `.upmarto/config.json` (via `@upmarto/sdk`).

---

## Captured events (v1)

| Activity | Event type |
|----------|------------|
| Document open / edit / save | `file_opened`, `file_modified` |
| New file | `file_created` |
| Tasks / terminal | `command_executed`, `test_*` |
| Extension activate | `agent_message` |

---

## Privacy

Data is sent only to your **local Upmarto server** (`127.0.0.1`). No mandatory cloud account. No third-party analytics in this extension.

---

## Links

- [Repository](https://github.com/mertcaliskanlinux/upmarto)
- [SDK guide](https://github.com/mertcaliskanlinux/upmarto/blob/main/agent-blackbox/docs/SDK.md)
- [API contract (v1)](https://github.com/mertcaliskanlinux/upmarto/blob/main/agent-blackbox/docs/API_CONTRACT.md)
- [Publishing notes](https://github.com/mertcaliskanlinux/upmarto/blob/main/agent-blackbox/docs/PUBLISHING.md)

## License

MIT — see [LICENSE](https://github.com/mertcaliskanlinux/upmarto/blob/main/agent-blackbox/LICENSE).
