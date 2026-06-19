# Registry onboarding & account hardening

Step 6 checklist before the definitive publish wave. **No API tokens belong in git.**

## Accounts to create

| Registry | Entity | URL |
|----------|--------|-----|
| npm | Organization `@upmarto` | https://www.npmjs.com/org/create |
| crates.io | User + API token | https://crates.io/settings/tokens |
| VS Marketplace | Publisher `upmarto` | https://marketplace.visualstudio.com/manage |
| OpenVSX (optional) | Namespace `upmarto` | https://open-vsx.org |

## One-time login (local machine only)

```bash
npm login                    # account with access to @upmarto
cargo login                  # paste crates.io token (stored in ~/.cargo/credentials.toml)
npx @vscode/vsce login upmarto
```

Tokens are stored in user home directories only:

- npm: `~/.npmrc` (never commit)
- cargo: `~/.cargo/credentials.toml` (never commit)
- vsce: VS Code Marketplace PAT (never commit)

## Verify before publish

```bash
cd agent-blackbox

# Cross-platform
node scripts/verify-registries.mjs

# Unix
./scripts/verify-registries.sh

# Windows
.\scripts\verify-registries.ps1
```

Strict mode (fail on warnings):

```bash
node scripts/verify-registries.mjs --strict
```

## Registry mode compile check

Before uploading cursor/vscode packages with `^0.1.0` deps:

```bash
node scripts/set-sdk-dependency.mjs registry --verify
```

This swaps deps, packs `@upmarto/sdk` locally, installs the tarball into consumers, and runs `build`/`compile`. On failure, reverts to `file:` links.

Restore local dev links:

```bash
node scripts/set-sdk-dependency.mjs local
```

## Publish wave (after verification passes)

See [PUBLISHING.md](./PUBLISHING.md):

1. `npm publish` — `@upmarto/sdk`
2. `cargo publish` — `upmarto-sdk`
3. `set-sdk-dependency.mjs registry` + `npm publish` — `@upmarto/cursor`
4. `cargo publish` — `upmarto-cli`
5. `vsce package` + marketplace upload — `upmarto-vscode`

## CI note

Registry verification is **local-only** (requires interactive logins). Do not add tokens to `.github/workflows/`.
