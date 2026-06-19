# Release assets

Pre-built binaries and checksums for GitHub Releases.

## Expected layout (per release tag)

```
assets/
  upmarto-cli-v0.1.0-x86_64-unknown-linux-gnu.tar.gz
  upmarto-cli-v0.1.0-x86_64-pc-windows-msvc.zip
  upmarto-cli-v0.1.0-aarch64-apple-darwin.tar.gz
  upmarto-server-v0.1.0-x86_64-unknown-linux-gnu.tar.gz
  upmarto-server-v0.1.0-x86_64-pc-windows-msvc.zip
  upmarto-server-v0.1.0-aarch64-apple-darwin.tar.gz
  upmarto-vscode-0.1.0.vsix
  SHA256SUMS
```

Binaries are produced by the release CI workflow (not committed to git).
