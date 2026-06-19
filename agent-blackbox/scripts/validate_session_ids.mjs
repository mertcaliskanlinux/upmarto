import { createHash } from "node:crypto";
import { resolve } from "node:path";
import { realpathSync } from "node:fs";
import { execSync } from "node:child_process";
import { deriveSessionId, resolveSessionId } from "../upmarto-sdk-ts/dist/session.js";

const workspace = resolve(".");

function tsNormalized(ws) {
  let canonical;
  try {
    const absolute = resolve(ws);
    canonical = realpathSync.native(absolute);
    if (process.platform === "win32" && !canonical.startsWith("\\\\?\\")) {
      if (/^[A-Za-z]:[\\/]/.test(canonical)) canonical = `\\\\?\\${canonical}`;
    }
  } catch {
    canonical = ws;
  }
  return canonical.replace(/\\/g, "/").toLowerCase();
}

const rustOut = execSync("cargo run -q -p upmarto-cli -- session", {
  cwd: workspace,
  encoding: "utf8",
});
const rustMatch = rustOut.match(/session_id:\s*(\S+)/);
const rustSession = rustMatch?.[1] ?? "MISSING";

const tsSession = resolveSessionId(workspace);
const normalized = tsNormalized(workspace);

console.log("workspace:", workspace);
console.log("ts normalized:", normalized);
console.log("rust session:", rustSession);
console.log("ts session:", tsSession);
console.log("MATCH:", rustSession === tsSession ? "PASS" : "FAIL");
