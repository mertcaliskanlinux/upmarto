## Summary

<!-- What does this PR change and why? -->

## Type of change

- [ ] Bug fix (non-breaking)
- [ ] New feature (non-breaking)
- [ ] Breaking change (requires API version bump / migration doc)
- [ ] Documentation only
- [ ] Release / packaging / CI

## Component

- [ ] Backend (`agent-blackbox/src/`)
- [ ] Rust SDK (`upmarto-sdk-rust/`)
- [ ] CLI (`upmarto-cli/`)
- [ ] TypeScript SDK (`upmarto-sdk-ts/`)
- [ ] Cursor (`upmarto-cursor/`)
- [ ] VS Code (`upmarto-vscode/`)
- [ ] UI (`ui/`)
- [ ] Tests / docs / other

## API contract

- [ ] No changes to frozen v1 API (`/event`, `/timeline`, `/explain`)
- [ ] API change included — `docs/API_CONTRACT.md` and version bump updated

## Testing

<!-- Commands run and results -->

```bash
cd agent-blackbox
cargo test --workspace
```

- [ ] Tests pass locally
- [ ] New tests added (if applicable)

## Checklist

- [ ] `cargo fmt` / `clippy` clean (Rust changes)
- [ ] TypeScript builds (`npm run build` in affected packages)
- [ ] CHANGELOG updated (user-facing changes)
- [ ] README / docs updated (if behavior or setup changed)

## Related issues

<!-- Fixes #123 -->
