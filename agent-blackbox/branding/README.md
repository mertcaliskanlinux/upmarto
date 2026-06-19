# Branding assets

Visual identity files for marketplaces and GitHub.

## Source of truth

Place the master icon at repo root as `icon.png` (128×128 PNG), then sync copies:

```powershell
# From repo root — re-run after changing icon.png
$icon = "icon.png"
Copy-Item $icon agent-blackbox/branding/icon.png
Copy-Item $icon agent-blackbox/branding/icon-128.png
Copy-Item $icon agent-blackbox/upmarto-vscode/media/icon-128.png
Copy-Item $icon agent-blackbox/assets/icon-128.png
```

## Files in this folder

| File | Size | Used by |
|------|------|---------|
| `icon.png` | 128×128 | Canonical copy (from repo root) |
| `icon-128.png` | 128×128 | VS Code Marketplace, OpenVSX, VSIX |
| `icon-256.png` | 256×256 | README headers, high-DPI |
| `social-preview.png` | 1280×640 | GitHub repository social preview |
| `gallery-banner.png` | 640×320 | VS Code Marketplace header (optional) |

See `docs/MARKETPLACE_ASSETS.md` for full requirements.
