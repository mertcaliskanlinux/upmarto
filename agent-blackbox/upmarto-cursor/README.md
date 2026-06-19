# @upmarto/cursor

[![CI](https://github.com/mertcaliskanlinux/upmarto/actions/workflows/ci.yml/badge.svg)](https://github.com/mertcaliskanlinux/upmarto/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://github.com/mertcaliskanlinux/upmarto/blob/main/agent-blackbox/LICENSE)
[![npm](https://img.shields.io/npm/v/@upmarto/cursor.svg)](https://www.npmjs.com/package/@upmarto/cursor)

Cursor IDE hooks for passive Upmarto event capture via `@upmarto/sdk`.

## Installation

```bash
npm install @upmarto/cursor
```

## Setup

1. Start the Upmarto backend and run `upmarto init` in your project.
2. Build the package: `npm run build`
3. Copy hooks into your project:

   ```bash
   cp node_modules/@upmarto/cursor/hooks.json .cursor/hooks.json
   ```

   Or reference the hook binary from `node_modules/@upmarto/cursor/dist/hook.js`.

4. Restart Cursor.

## Captured events

File edits, shell commands, agent responses, and tool use map to frozen v1 event types (`file_modified`, `command_executed`, `test_failed`, etc.).

## Documentation

- [SDK guide](https://github.com/mertcaliskanlinux/upmarto/blob/main/agent-blackbox/docs/SDK.md)

## License

MIT — see [LICENSE](../LICENSE).
