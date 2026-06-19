# Publishing sequence

Registry-ready publish order for the Upmarto monorepo. Local development uses `file:../upmarto-sdk-ts`; registry publish uses `^0.1.0`.

## Dependency modes

| Mode | `@upmarto/sdk` (npm) | `upmarto-sdk` (Rust) |
|------|----------------------|----------------------|
| **Local dev** (default) | `file:../upmarto-sdk-ts` | `path = "../upmarto-sdk-rust", version = "0.1.0"` |
| **Registry publish** | `^0.1.0` | `version = "0.1.0"` (path removed automatically by `cargo publish`) |

Switch npm consumers:

```bash
cd agent-blackbox
node scripts/verify-registries.mjs          # pre-flight login & dry-run checks
node scripts/set-sdk-dependency.mjs registry --verify   # swap + compile verify
node scripts/set-sdk-dependency.mjs registry            # swap only
node scripts/set-sdk-dependency.mjs local             # restore monorepo file: links
```

See [REGISTRY_ONBOARDING.md](./REGISTRY_ONBOARDING.md) for account setup.

## Publish order

1. **npm** — `@upmarto/sdk@0.1.0`
2. **crates.io** — `upmarto-sdk@0.1.0`
3. **npm** — `@upmarto/cursor@0.1.0` (after step 1 + `set-sdk-dependency.mjs registry`)
4. **crates.io** — `upmarto-cli@0.1.0` (after step 2)
5. **VSIX** — `upmarto-vscode` (after step 1; `npm run compile` + `vsce package`)
6. **GitHub Release** — binaries + VSIX + `SHA256SUMS`

## VS Code extension

- Marketplace icon: `media/icon-128.png` (bundled in VSIX)
- Shared branding: `branding/icon-128.png`
- Before `vsce package`: run `set-sdk-dependency.mjs registry` and `npm install` in `upmarto-vscode/`

## Rust CLI

`upmarto-cli/Cargo.toml` already declares:

```toml
upmarto-sdk = { path = "../upmarto-sdk-rust", version = "0.1.0" }
```

Publish `upmarto-sdk` first, then `cargo publish -p upmarto-cli`.
