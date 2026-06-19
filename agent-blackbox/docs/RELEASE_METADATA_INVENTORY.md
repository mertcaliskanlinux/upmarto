# Release metadata inventory

Canonical repository: `https://github.com/mertcaliskanlinux/upmarto`  
License: MIT  
Version: `0.1.0` (all packages)

## Rust SDK — `upmarto-sdk`

| Field | Value |
|-------|-------|
| Crate name | `upmarto-sdk` |
| Path | `upmarto-sdk-rust/` |
| Version | `0.1.0` |
| Registry | crates.io |
| README | `upmarto-sdk-rust/README.md` |
| LICENSE | `../LICENSE` via `license-file` |
| Repository | `https://github.com/mertcaliskanlinux/upmarto` |
| Homepage | `https://github.com/mertcaliskanlinux/upmarto` |
| Documentation | `https://docs.rs/upmarto-sdk` |
| Keywords | `ai`, `agents`, `debugging`, `events`, `timeline` |
| Categories | `development-tools`, `api-bindings` |
| Authors | Upmarto Contributors |
| Publish order | **1** (dependency for CLI) |

## Rust CLI — `upmarto-cli`

| Field | Value |
|-------|-------|
| Crate name | `upmarto-cli` |
| Binary | `upmarto-cli` |
| Path | `upmarto-cli/` |
| Version | `0.1.0` |
| Registry | crates.io |
| README | `upmarto-cli/README.md` |
| LICENSE | `../LICENSE` via `license-file` |
| Repository | `https://github.com/mertcaliskanlinux/upmarto` |
| Homepage | `https://github.com/mertcaliskanlinux/upmarto` |
| Documentation | SDK guide (linked in Cargo.toml) |
| Keywords | `ai`, `agents`, `cli`, `debugging`, `upmarto` |
| Categories | `command-line-utilities`, `development-tools` |
| Depends on | `upmarto-sdk = "0.1.0"` |
| Publish order | **2** |

## TypeScript SDK — `@upmarto/sdk`

| Field | Value |
|-------|-------|
| Package name | `@upmarto/sdk` |
| Path | `upmarto-sdk-ts/` |
| Version | `0.1.0` |
| Registry | npm (`@upmarto` scope) |
| README | `upmarto-sdk-ts/README.md` |
| LICENSE | `upmarto-sdk-ts/LICENSE` |
| Repository directory | `agent-blackbox/upmarto-sdk-ts` |
| Main / types | `dist/index.js`, `dist/index.d.ts` |
| Exports | ESM `.` entry |
| Files whitelist | `dist`, `README.md`, `LICENSE` |
| Engines | Node `>=18` |
| Publish order | **1** (npm) |

## Cursor — `@upmarto/cursor`

| Field | Value |
|-------|-------|
| Package name | `@upmarto/cursor` |
| Path | `upmarto-cursor/` |
| Version | `0.1.0` |
| Registry | npm |
| README | `upmarto-cursor/README.md` |
| LICENSE | `upmarto-cursor/LICENSE` |
| Bin | `upmarto-cursor-hook` |
| Files whitelist | `dist`, `hooks.json`, `README.md`, `LICENSE` |
| Depends on | `@upmarto/sdk` (monorepo: `file:../upmarto-sdk-ts`) |
| Publish order | **2** (after SDK on npm) |

## VS Code — `upmarto-vscode`

| Field | Value |
|-------|-------|
| Extension ID | `upmarto.upmarto-vscode` |
| Display name | Upmarto |
| Path | `upmarto-vscode/` |
| Version | `0.1.0` |
| Publisher | `upmarto` |
| Marketplaces | VS Marketplace, OpenVSX |
| README | `upmarto-vscode/README.md` |
| LICENSE | `upmarto-vscode/LICENSE` |
| Categories | `Other`, `Debuggers` |
| Activation | `onStartupFinished` |
| VSIX output | `upmarto-vscode-{version}.vsix` |
| Depends on | `@upmarto/sdk` (monorepo: `file:../upmarto-sdk-ts`) |

## Out of scope (not publishable as crates.io root)

| Component | Path | Distribution channel |
|-----------|------|----------------------|
| Backend server | workspace root `upmarto` | GitHub Release binaries only |
| Web UI | `ui/` | Bundled with server / static deploy |
