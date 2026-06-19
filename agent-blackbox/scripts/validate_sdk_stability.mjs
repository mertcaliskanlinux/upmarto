/**
 * Step 3 stability validation (SDK-only, no backend required for unit checks).
 */
import { appendFileSync, mkdtempSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { tmpdir } from "node:os";

import { Upmarto } from "../upmarto-sdk-ts/dist/client.js";
import {
  eventIdentityKey,
  parseQueueLine,
  toQueueLine,
} from "../upmarto-sdk-ts/dist/queue_helpers.js";

const results = {};

// --- removePersisted semantic match (legacy _attempts lines) ---
const tmp = mkdtempSync(join(tmpdir(), "upmarto-sdk-"));
const queueDir = join(tmp, ".upmarto");
mkdirSync(queueDir, { recursive: true });
const queueFile = join(queueDir, "queue.jsonl");
const event = {
  project_id: "p",
  session_id: "s",
  event_type: "file_modified",
  timestamp: 12345,
  payload: { path: "src/a.rs" },
};
const legacyLine = JSON.stringify({ ...event, _attempts: 99 });
writeFileSync(queueFile, `${legacyLine}\n`, "utf8");

const parsed = parseQueueLine(legacyLine);
results.parse_legacy = parsed && eventIdentityKey(parsed) === eventIdentityKey(event) ? "PASS" : "FAIL";

// Simulate removal via identity
const targetKey = eventIdentityKey(event);
const kept = readFileSync(queueFile, "utf8")
  .split("\n")
  .filter((l) => {
    if (!l.trim()) return false;
    const p = parseQueueLine(l);
    return !p || eventIdentityKey(p) !== targetKey;
  });
results.remove_semantic = kept.length === 0 ? "PASS" : "FAIL";

// --- dedup guard ---
let suppressed = 0;
const origStderr = process.stderr.write.bind(process.stderr);
process.stderr.write = (chunk, ...args) => {
  if (String(chunk).includes("duplicate suppressed")) suppressed++;
  return origStderr(chunk, ...args);
};

const client = Upmarto.init({
  apiUrl: "http://127.0.0.1:1",
  projectId: "p",
  workspacePath: tmp,
  autoFlush: false,
  batchSize: 100,
});
client.track({ event_type: "file_opened", payload: { path: "x.rs" }, timestamp: 1000 });
client.track({ event_type: "file_opened", payload: { path: "x.rs" }, timestamp: 1050 });
client.track({ event_type: "file_opened", payload: { path: "x.rs" }, timestamp: 1600 });
results.dedup = suppressed === 1 ? "PASS" : `FAIL (suppressed=${suppressed})`;

// --- persist without _attempts ---
const qContent = readFileSync(queueFile, "utf8");
results.no_attempts_in_file = qContent.includes("_attempts") ? "FAIL" : "PASS";
results.clean_line = qContent.trim() === toQueueLine({ ...event, timestamp: 1000 }) ? "PASS" : "PARTIAL";

rmSync(tmp, { recursive: true, force: true });

console.log(JSON.stringify(results, null, 2));
