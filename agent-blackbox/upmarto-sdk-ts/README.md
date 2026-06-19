# @upmarto/sdk

[![CI](https://github.com/mertcaliskanlinux/upmarto/actions/workflows/ci.yml/badge.svg)](https://github.com/mertcaliskanlinux/upmarto/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://github.com/mertcaliskanlinux/upmarto/blob/main/agent-blackbox/LICENSE)
[![npm](https://img.shields.io/npm/v/@upmarto/sdk.svg)](https://www.npmjs.com/package/@upmarto/sdk)

TypeScript SDK for the Upmarto v1 API — Node.js, Cursor hooks, and VS Code extensions.

## Installation

```bash
npm install @upmarto/sdk
```

## Usage

```typescript
import { Upmarto } from "@upmarto/sdk";

const client = await Upmarto.fromWorkspace();
client.track({
  event_type: "file_modified",
  payload: { path: "src/main.rs" },
});
await client.flush();
```

## Features

- ESM with TypeScript declarations
- Local queue and retry (mirrors Rust SDK)
- Workspace config: `.upmarto/config.json`
- Session ID derivation aligned with Rust SDK

## Build (monorepo)

```bash
npm install
npm run build
```

## Documentation

- [SDK guide](https://github.com/mertcaliskanlinux/upmarto/blob/main/agent-blackbox/docs/SDK.md)
- [API contract](https://github.com/mertcaliskanlinux/upmarto/blob/main/agent-blackbox/docs/API_CONTRACT.md)

## License

MIT — see [LICENSE](../LICENSE).
