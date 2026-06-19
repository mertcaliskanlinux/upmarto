import { createHash } from "node:crypto";
import { realpathSync } from "node:fs";
import { resolve } from "node:path";

export function localDateKey(date = new Date()): string {
  const y = date.getFullYear();
  const m = String(date.getMonth() + 1).padStart(2, "0");
  const d = String(date.getDate()).padStart(2, "0");
  return `${y}-${m}-${d}`;
}

/**
 * Match Rust `Path::canonicalize()` byte-for-byte before slash/case normalization.
 * On Windows, Rust emits extended-length paths (`\\?\C:\...` → `//?/c:/...`).
 */
function canonicalizeWorkspace(workspace: string): string {
  try {
    const absolute = resolve(workspace);
    let canonical = realpathSync.native(absolute);
    if (process.platform === "win32" && !canonical.startsWith("\\\\?\\")) {
      if (/^[A-Za-z]:[\\/]/.test(canonical)) {
        canonical = `\\\\?\\${canonical}`;
      } else if (canonical.startsWith("\\\\")) {
        canonical = `\\\\?\\UNC\\${canonical.slice(2)}`;
      }
    }
    return canonical;
  } catch {
    return workspace;
  }
}

export function deriveSessionId(workspace: string, dateKey = localDateKey()): string {
  const normalized = canonicalizeWorkspace(workspace).replace(/\\/g, "/").toLowerCase();
  const hash = createHash("sha256")
    .update(`${normalized}:${dateKey}`)
    .digest("hex")
    .slice(0, 32);
  return `${hash.slice(0, 8)}-${hash.slice(8, 12)}-${hash.slice(12, 16)}-${hash.slice(16, 20)}-${hash.slice(20, 32)}`;
}

export function resolveSessionId(workspace: string): string {
  return deriveSessionId(workspace, localDateKey());
}
