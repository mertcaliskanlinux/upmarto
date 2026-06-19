# Marketplace asset requirements

Assets required before public listing. **None of the binary image files exist yet** — only directory scaffolding and this checklist.

## Required assets

### VS Code Marketplace

| Asset | Path (planned) | Spec | Status |
|-------|----------------|------|--------|
| Extension icon | `branding/icon-128.png` | 128×128 PNG | ❌ Missing |
| Gallery banner | `branding/gallery-banner.png` | 640×320 PNG | ❌ Missing |
| README (long description) | `upmarto-vscode/README.md` | Markdown | ✅ Created |
| LICENSE | `upmarto-vscode/LICENSE` | MIT text | ✅ Created |
| Screenshots (≥1) | `screenshots/01-timeline.png` … | 1280×800 recommended | ❌ Missing |
| Publisher account | `upmarto` on marketplace | Registered org | ❓ Unverified |

### OpenVSX

| Asset | Path (planned) | Spec | Status |
|-------|----------------|------|--------|
| Extension icon | `branding/icon-128.png` | 128×128 PNG (same as VS Code) | ❌ Missing |
| README | `upmarto-vscode/README.md` | Markdown | ✅ Created |
| LICENSE | `upmarto-vscode/LICENSE` | MIT | ✅ Created |
| VSIX artifact | `assets/upmarto-vscode-0.1.0.vsix` | Built at release | ⚠️ Build path verified; SDK bundle TBD |
| Namespace | `upmarto` on open-vsx.org | Registered | ❓ Unverified |

### npm (`@upmarto/sdk`, `@upmarto/cursor`)

| Asset | Path | Status |
|-------|------|--------|
| Package README | `upmarto-sdk-ts/README.md`, `upmarto-cursor/README.md` | ✅ Created |
| LICENSE in tarball | `LICENSE` per package | ✅ Created |
| npm org `@upmarto` | registry.npmjs.org | ❓ Unverified |
| Keyword metadata | `package.json` `keywords` | ✅ Added |

### crates.io (`upmarto-sdk`, `upmarto-cli`)

| Asset | Path | Status |
|-------|------|--------|
| Crate README | `README.md` per crate | ✅ Created |
| LICENSE | Root `LICENSE` via `license-file` | ✅ Created |
| Keywords / categories | `Cargo.toml` | ✅ Added |
| docs.rs | Auto from crate | ⏳ After publish |

### GitHub

| Asset | Path (planned) | Spec | Status |
|-------|----------------|------|--------|
| Social preview | `branding/social-preview.png` | 1280×640 | ❌ Missing |
| Release notes | `CHANGELOG.md` | Keep a Changelog | ✅ Created |
| Release binaries | `assets/` (CI artifacts) | per-target archives | ❌ CI not configured |
| Public repository | `github.com/mertcaliskanlinux/upmarto` | Git remote | ❓ Unverified |

## Screenshot capture list

| # | File | Content to capture |
|---|------|-------------------|
| 1 | `screenshots/01-timeline.png` | UI timeline with session events |
| 2 | `screenshots/02-explain.png` | Explain output (root cause visible) |
| 3 | `screenshots/03-cli-workflow.png` | `upmarto workflow` + `upmarto explain` terminal |
| 4 | `screenshots/04-vscode-capture.png` | VS Code + Upmarto output channel |
| 5 | `screenshots/05-config.png` | `upmarto init` + Current Session output |

## VSIX build notes

- Command: `npx @vscode/vsce package` from `upmarto-vscode/`
- **Blocker:** `file:../upmarto-sdk-ts` dependency prevents bundling until `@upmarto/sdk` is published or vendored into the extension
- **Blocker:** `icon` field not set until `branding/icon-128.png` exists
