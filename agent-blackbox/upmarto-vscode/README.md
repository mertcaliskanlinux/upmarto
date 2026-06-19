# Upmarto — VS Code Extension

[![CI](https://github.com/mertcaliskanlinux/upmarto/actions/workflows/ci.yml/badge.svg)](https://github.com/mertcaliskanlinux/upmarto/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://github.com/mertcaliskanlinux/upmarto/blob/main/agent-blackbox/LICENSE)
[![VS Marketplace](https://img.shields.io/badge/VS%20Marketplace-Upmarto-blue?logo=visualstudiocode)](https://marketplace.visualstudio.com/search?term=upmarto)

Capture developer and AI agent activity into [Upmarto](https://github.com/mertcaliskanlinux/upmarto) using `@upmarto/sdk`.

## Requirements

- Upmarto backend running locally
- Project initialized with `upmarto init` (`.upmarto/config.json`)

## Installation

- **Development:** open this folder in VS Code and press F5.
- **Release:** install the `.vsix` from GitHub Releases.

## Configuration

| Setting | Default | Description |
|---------|---------|-------------|
| `upmarto.enabled` | `true` | Toggle passive capture |

API URL and queue settings come from `.upmarto/config.json` (same as Cursor hooks).

## Captured events

| VS Code API | Event type |
|-------------|------------|
| Document open / edit / save | `file_opened`, `file_modified` |
| File create | `file_created` |
| Tasks / terminal | `command_executed`, `test_*` |
| Extension activate | `agent_message` |

## Build

```bash
cd ../upmarto-sdk-ts && npm install && npm run build
cd ../upmarto-vscode && npm install && npm run build
```

## License

MIT — see [LICENSE](../LICENSE).
