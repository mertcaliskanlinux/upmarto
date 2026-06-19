# upmarto-cli

[![CI](https://github.com/mertcaliskanlinux/upmarto/actions/workflows/ci.yml/badge.svg)](https://github.com/mertcaliskanlinux/upmarto/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://github.com/mertcaliskanlinux/upmarto/blob/main/agent-blackbox/LICENSE)
[![crates.io](https://img.shields.io/crates/v/upmarto-cli.svg)](https://crates.io/crates/upmarto-cli)

Command-line interface for Upmarto — initialize projects, track events, run workflows, and explain sessions.

## Installation

```bash
cargo install upmarto-cli
```

From source:

```bash
cargo install --path upmarto-cli
```

The installed binary is `upmarto-cli`. Alias or symlink to `upmarto` if desired.

## Quick start

```bash
# Start backend first (see main README), then:
upmarto-cli init
upmarto-cli workflow
upmarto-cli explain
```

## Commands

| Command | Description |
|---------|-------------|
| `init` | Discover backend, write `.upmarto/config.json` |
| `workflow` | Isolated 6-event bug-fix scenario |
| `explain [session_id]` | WHY engine (uses active session when omitted) |
| `track` | Track a single event |
| `session` | Show current session ID |
| `flush` | Flush pending queue |
| `demo` | Quick 4-event demo scenario |

## Documentation

- [SDK guide](https://github.com/mertcaliskanlinux/upmarto/blob/main/agent-blackbox/docs/SDK.md)

## License

MIT — see [LICENSE](../LICENSE).
