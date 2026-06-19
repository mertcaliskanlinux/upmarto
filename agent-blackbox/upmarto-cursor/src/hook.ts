#!/usr/bin/env node
import { resolve } from "node:path";

import { defaultHookResponse, trackHookEvent } from "./event_tracker.js";

function readStdin(): Promise<string> {
  return new Promise((resolvePromise) => {
    const chunks: Buffer[] = [];
    process.stdin.on("data", (c) => chunks.push(c));
    process.stdin.on("end", () => resolvePromise(Buffer.concat(chunks).toString("utf8")));
    process.stdin.resume();
  });
}

function workspacePath(): string {
  return resolve(
    process.env.CURSOR_PROJECT_DIR ??
      process.env.VSCODE_CWD ??
      process.env.INIT_CWD ??
      process.cwd(),
  );
}

async function main(): Promise<void> {
  const hookName = process.argv[2] ?? "unknown";
  const raw = await readStdin();
  let input: Record<string, unknown> = {};

  if (raw.trim()) {
    try {
      input = JSON.parse(raw) as Record<string, unknown>;
    } catch {
      input = { raw: raw.slice(0, 500) };
    }
  }

  try {
    trackHookEvent({ hookName, workspacePath: workspacePath(), input });
  } catch {
    // Never fail the hook
  }

  const response = defaultHookResponse(hookName);
  if (Object.keys(response).length > 0) {
    process.stdout.write(JSON.stringify(response));
  }
}

main().catch(() => process.exit(0));
