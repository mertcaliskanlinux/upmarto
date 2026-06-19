# Rename: Agent Blackbox → Upmarto

**Agent Blackbox** is deprecated. The product is now **Upmarto**.

| Old | New |
|-----|-----|
| Agent Blackbox | Upmarto |
| `agent-blackbox` (package/binary) | `upmarto` |
| `agent_blackbox` (Rust crate) | `upmarto` |
| `BLACKBOX_URL` | `UPMARTO_URL` (legacy `BLACKBOX_URL` still accepted) |
| `.blackbox/` session state | `.upmarto/` |
| `blackbox-hook` / `blackbox-emit` | `upmarto-hook` / `upmarto-emit` |
| Repository folder `agent-blackbox/` | `upmarto/` (rename manually if IDE has the folder open) |

**Unchanged (v1 API contract):**
- All HTTP endpoints (`/event`, `/timeline`, `/explain`, etc.)
- Event types and payload schemas
- Database schema (`events.log`, `metadata.db`)
- Reasoning engine behavior

**Tagline:** Memory and Reasoning for AI Agents
