# Release Foundation Report

**Sprint:** Distribution Step 1 — Release Foundation  
**Date:** 2026-06-19  
**Scope:** Packaging metadata and release scaffolding only. No backend, SDK logic, API, session, queue, explain, storage, or test changes.

---

## Summary

Release foundation artifacts are in place for all five publishable components. `@upmarto/sdk` and `upmarto-sdk` pass dry-run packaging. Remaining blockers are external (registry accounts, git remote, image assets) and structural (`file:` npm deps, CLI publish order).

**Overall foundation readiness: 62/100** (up from 38/100 pre-foundation)

---

## Created files

| File | Purpose |
|------|---------|
| `LICENSE` | MIT license (root) |
| `CHANGELOG.md` | Keep a Changelog for v0.1.0 |
| `assets/README.md` | GitHub Release binary layout |
| `branding/README.md` | Icon/banner asset spec |
| `screenshots/README.md` | Marketplace screenshot capture list |
| `upmarto-sdk-rust/README.md` | crates.io README |
| `upmarto-cli/README.md` | crates.io README |
| `upmarto-sdk-ts/README.md` | npm README |
| `upmarto-sdk-ts/LICENSE` | npm tarball license |
| `upmarto-cursor/README.md` | npm README |
| `upmarto-cursor/LICENSE` | npm tarball license |
| `upmarto-vscode/README.md` | Marketplace long description |
| `upmarto-vscode/LICENSE` | Marketplace license |
| `docs/RELEASE_METADATA_INVENTORY.md` | Per-package metadata table |
| `docs/MARKETPLACE_ASSETS.md` | Required marketplace assets checklist |
| `RELEASE_FOUNDATION_REPORT.md` | This report |

---

## Modified files

| File | Changes |
|------|---------|
| `upmarto-sdk-rust/Cargo.toml` | authors, readme, repository, homepage, documentation, keywords, categories, license-file |
| `upmarto-cli/Cargo.toml` | Full metadata + `upmarto-sdk` version requirement `0.1.0` |
| `upmarto-sdk-ts/package.json` | license, author, repository, homepage, bugs, keywords, files, prepublishOnly |
| `upmarto-cursor/package.json` | Removed `private`, metadata, files whitelist, prepublishOnly |
| `upmarto-vscode/package.json` | Removed `private`, metadata, keywords, categories (`Debuggers`), improved description |

---

## Release metadata inventory

See [docs/RELEASE_METADATA_INVENTORY.md](docs/RELEASE_METADATA_INVENTORY.md).

| Package | Name | Version | Registry |
|---------|------|---------|----------|
| Rust SDK | `upmarto-sdk` | 0.1.0 | crates.io |
| Rust CLI | `upmarto-cli` | 0.1.0 | crates.io |
| TypeScript SDK | `@upmarto/sdk` | 0.1.0 | npm |
| Cursor | `@upmarto/cursor` | 0.1.0 | npm |
| VS Code | `upmarto-vscode` | 0.1.0 | VS Marketplace / OpenVSX |

Canonical URLs (configured, not verified live):

- Repository: `https://github.com/mertcaliskanlinux/upmarto`
- Homepage: `https://github.com/mertcaliskanlinux/upmarto`
- Issues: `https://github.com/mertcaliskanlinux/upmarto/issues`

---

## Validation results

| Check | Result |
|-------|--------|
| `cargo package -p upmarto-sdk --allow-dirty` | **PASS** — 12 files, verify OK (warning: `license` + `license-file` redundant for MIT) |
| `cargo package -p upmarto-cli --allow-dirty` | **FAIL** — `upmarto-sdk` not on crates.io yet (expected until SDK published first) |
| `npm pack --dry-run` (`@upmarto/sdk`) | **PASS** — 17 files incl. README + LICENSE |
| `npm pack --dry-run` (`@upmarto/cursor`) | **PASS** — 6 files, no `src/` leakage |
| VSIX build | Not re-run this step (unchanged: requires published SDK or bundle step) |

---

## Marketplace asset requirements

See [docs/MARKETPLACE_ASSETS.md](docs/MARKETPLACE_ASSETS.md).

### Still missing (binary assets)

| Asset | Channel |
|-------|---------|
| `branding/icon-128.png` | VS Code, OpenVSX |
| `branding/gallery-banner.png` | VS Code Marketplace |
| `branding/social-preview.png` | GitHub |
| `screenshots/01-timeline.png` … `05-config.png` | Marketplaces, README |

### Created (text/metadata)

| Asset | Status |
|-------|--------|
| npm package READMEs | ✅ |
| crates.io READMEs | ✅ |
| VS Code README | ✅ |
| LICENSE (all packages) | ✅ |
| CHANGELOG | ✅ |
| Keywords / categories | ✅ |

---

## Remaining blockers

| # | Blocker | Affects | Resolution |
|---|---------|---------|------------|
| 1 | Public git repo not initialized / remote unverified | All | Push to `github.com/mertcaliskanlinux/upmarto` |
| 2 | `upmarto-sdk` not on crates.io | CLI `cargo package` | Publish SDK first |
| 3 | `file:../upmarto-sdk-ts` in cursor/vscode | npm publish, VSIX | Switch to `^0.1.0` after SDK on npm; or bundle SDK in VSIX |
| 4 | Binary name `upmarto-cli` vs UX `upmarto` | CLI distribution | Rename `[[bin]]` or document alias |
| 5 | No branding images | Marketplaces, GitHub | Design + commit to `branding/` |
| 6 | No screenshots | Marketplaces | Capture per `screenshots/README.md` |
| 7 | Publisher accounts unverified | npm `@upmarto`, VS `upmarto`, OpenVSX | Register orgs |
| 8 | No release CI workflow | GitHub assets | Add `.github/workflows/release.yml` |
| 9 | Backend workspace crate bloat | Accidental publish | Add `.cargoignore`; do not publish root `upmarto` crate |
| 10 | VSIX runtime SDK | Extension install | Bundle or npm dep before marketplace submit |

---

## Publish readiness by package

| Package | Score | Status |
|---------|-------|--------|
| **upmarto-sdk** (Rust) | **78/100** | Metadata complete; `cargo package` passes; publish when repo + crates.io token ready |
| **upmarto-cli** (Rust) | **55/100** | Metadata complete; blocked until SDK on crates.io; binary naming TBD |
| **@upmarto/sdk** (npm) | **82/100** | `npm pack` clean; needs npm org + public repo |
| **@upmarto/cursor** (npm) | **68/100** | Tarball clean; `file:` dep blocks external install until SDK published |
| **upmarto-vscode** | **58/100** | Metadata + LICENSE; missing icon, screenshots, SDK bundle |
| **Release CI / GitHub** | **25/100** | Structure only; no automated artifacts |
| **Marketplace visuals** | **15/100** | Directories + specs; no image files |

**Weighted overall: 62/100**

---

## Recommended publish order

1. Initialize public repository and push foundation commit  
2. `cargo publish -p upmarto-sdk`  
3. `npm publish --access public` for `@upmarto/sdk`  
4. `cargo publish -p upmarto-cli`  
5. `npm publish` for `@upmarto/cursor` (update dep to `^0.1.0`)  
6. Bundle VSIX with SDK → VS Marketplace + OpenVSX  
7. GitHub Release with cross-compiled binaries + VSIX + `SHA256SUMS`

---

## Release directory structure

```
agent-blackbox/
├── LICENSE
├── CHANGELOG.md
├── RELEASE_FOUNDATION_REPORT.md
├── assets/          # CI-produced binaries (documented, empty)
├── branding/        # Icons, banners (documented, empty)
├── screenshots/     # Marketplace captures (documented, empty)
└── docs/
    ├── RELEASE_METADATA_INVENTORY.md
    └── MARKETPLACE_ASSETS.md
```

---

## Next sprint (Distribution Step 2 — not in scope)

- Add `.cargoignore` for workspace root  
- Release GitHub Actions workflow  
- Create `branding/icon-128.png` and wire `icon` in `upmarto-vscode/package.json`  
- Capture screenshots  
- Swap `file:` deps to semver for publish builds  
- Register npm / VS / OpenVSX publisher accounts

**Nothing was published in this step.**
