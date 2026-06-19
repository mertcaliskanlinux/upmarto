# Contributing to Upmarto

Thank you for your interest in contributing. Upmarto is a local-first platform for capturing AI agent activity and explaining session behavior deterministically.

## Development setup

1. **Clone the repository**

   ```bash
   git clone git@github.com:mertcaliskanlinux/upmarto.git
   cd upmarto/agent-blackbox
   ```

2. **Start the backend**

   ```bash
   cargo run
   ```

3. **Initialize a workspace**

   ```bash
   cargo run -p upmarto-cli -- init
   ```

4. **TypeScript packages** (Cursor / VS Code / SDK)

   ```bash
   cd upmarto-sdk-ts && npm install && npm run build
   cd ../upmarto-cursor && npm install && npm run build
   cd ../upmarto-vscode && npm install && npm run build
   ```

## Project structure

All product code lives under `agent-blackbox/`:

- `src/` — Rust backend (frozen v1 API)
- `upmarto-sdk-rust/` — Rust SDK
- `upmarto-cli/` — CLI
- `upmarto-sdk-ts/` — TypeScript SDK
- `upmarto-cursor/` — Cursor hooks
- `upmarto-vscode/` — VS Code extension
- `ui/` — Web timeline UI
- `tests/` — Integration, e2e, and scenario tests
- `docs/` — API contract, SDK guide, deployment

## Making changes

### API contract

The v1 API is frozen. Changes to `/event`, `/timeline`, `/explain`, or response schemas require a version bump documented in `docs/API_CONTRACT.md`. Do not merge breaking API changes without explicit maintainer approval.

### Rust

```bash
cd agent-blackbox
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test --workspace
```

### TypeScript

```bash
cd agent-blackbox/upmarto-sdk-ts && npm run build
```

### Pull requests

1. Fork the repository and create a branch from `main`.
2. Keep changes focused — one concern per PR when possible.
3. Add or update tests for behavior changes.
4. Ensure CI passes (see `.github/workflows/ci.yml`).
5. Fill out the [pull request template](.github/pull_request_template.md).

## Reporting issues

Use [GitHub Issues](https://github.com/mertcaliskanlinux/upmarto/issues) with the appropriate template:

- **Bug report** — reproducible failures
- **Feature request** — enhancements (non-breaking preferred)

For security vulnerabilities, see [SECURITY.md](SECURITY.md). Do not open public issues for security reports.

## Code style

- **Rust:** `rustfmt` defaults, idiomatic error handling with `thiserror`
- **TypeScript:** strict mode, ESM (`NodeNext`)
- **Commits:** clear, imperative subject lines (e.g. `fix: flush queue on reconnect`)

## Questions

Open a [Discussion](https://github.com/mertcaliskanlinux/upmarto/discussions) or an issue labeled `question` if unsure where a change belongs.
