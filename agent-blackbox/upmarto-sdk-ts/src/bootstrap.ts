import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";

import { writeProjectConfig } from "./config.js";

export const NOT_CONFIGURED_MSG =
  "Upmarto backend not configured. Run: upmarto init";

const PROBE_TIMEOUT_MS = 400;
const SCAN_START = 59_000;
const SCAN_END = 60_000;
const SCAN_BATCH = 64;

export interface RuntimeFile {
  api_url: string;
  port: number;
  detected_at?: string;
}

export function runtimeConfigPath(workspace: string): string {
  return join(resolve(workspace), ".upmarto", "runtime.json");
}

export function readDotenvPort(workspace: string): number | null {
  const envPath = join(resolve(workspace), ".env");
  if (!existsSync(envPath)) return null;
  for (const line of readFileSync(envPath, "utf8").split("\n")) {
    const trimmed = line.trim();
    if (trimmed.startsWith("APP_PORT=")) {
      const port = Number.parseInt(
        trimmed.slice("APP_PORT=".length).trim().replace(/^"|"$/g, ""),
        10,
      );
      return Number.isFinite(port) ? port : null;
    }
  }
  return null;
}

export async function probeBackend(apiUrl: string): Promise<boolean> {
  const url = `${apiUrl.replace(/\/$/, "")}/config`;
  const controller = new AbortController();
  const timer = setTimeout(() => controller.abort(), PROBE_TIMEOUT_MS);
  try {
    const res = await fetch(url, { signal: controller.signal });
    if (!res.ok) return false;
    const body = (await res.json()) as { api_version?: string };
    return body.api_version === "v1";
  } catch {
    return false;
  } finally {
    clearTimeout(timer);
  }
}

export function readRuntimeFile(workspace: string): RuntimeFile | null {
  const path = runtimeConfigPath(workspace);
  if (!existsSync(path)) return null;
  try {
    return JSON.parse(readFileSync(path, "utf8")) as RuntimeFile;
  } catch {
    return null;
  }
}

export function writeRuntimeFile(
  workspace: string,
  apiUrl: string,
  port: number,
): void {
  const dir = join(resolve(workspace), ".upmarto");
  mkdirSync(dir, { recursive: true });
  const runtime: RuntimeFile = {
    api_url: apiUrl.replace(/\/$/, ""),
    port,
    detected_at: new Date().toISOString(),
  };
  writeFileSync(runtimeConfigPath(workspace), JSON.stringify(runtime, null, 2), "utf8");
}

function parsePort(apiUrl: string): number {
  const match = apiUrl.match(/:(\d+)\/?$/);
  return match ? Number.parseInt(match[1], 10) : 0;
}

async function scanLocalPorts(): Promise<string | null> {
  const ports: number[] = [];
  for (let p = SCAN_START; p <= SCAN_END; p++) ports.push(p);

  for (let i = 0; i < ports.length; i += SCAN_BATCH) {
    const chunk = ports.slice(i, i + SCAN_BATCH);
    const results = await Promise.all(
      chunk.map(async (port) => {
        const url = `http://127.0.0.1:${port}`;
        return (await probeBackend(url)) ? url : null;
      }),
    );
    const hit = results.find((r) => r !== null);
    if (hit) return hit;
  }
  return null;
}

export async function discoverBackend(workspace: string): Promise<string> {
  const runtime = readRuntimeFile(workspace);
  if (runtime?.api_url && (await probeBackend(runtime.api_url))) {
    return runtime.api_url.replace(/\/$/, "");
  }

  const envPort = readDotenvPort(workspace);
  if (envPort) {
    const url = `http://127.0.0.1:${envPort}`;
    if (await probeBackend(url)) return url;
  }

  const scanned = await scanLocalPorts();
  if (scanned) return scanned;

  throw new Error(
    `No running Upmarto backend found on 127.0.0.1:${SCAN_START}-${SCAN_END}. ` +
      "Start the server (`cargo run`) then run: upmarto init",
  );
}

export async function validateWorkspaceAccess(
  workspace: string,
  apiUrl: string,
): Promise<void> {
  if (!apiUrl) {
    throw new Error(NOT_CONFIGURED_MSG);
  }

  const envOverride = Boolean(
    process.env.UPMARTO_URL?.trim() || process.env.BLACKBOX_URL?.trim(),
  );
  const projectConfig = join(resolve(workspace), ".upmarto", "config.json");

  if (!envOverride && !existsSync(projectConfig)) {
    throw new Error(NOT_CONFIGURED_MSG);
  }

  if (!envOverride && !(await probeBackend(apiUrl))) {
    throw new Error(
      `Upmarto backend not reachable at ${apiUrl}. Run: upmarto init`,
    );
  }
}

export async function bootstrapWorkspace(
  workspace: string,
  explicitApiUrl?: string,
): Promise<string> {
  let apiUrl = explicitApiUrl?.replace(/\/$/, "");
  if (!apiUrl) {
    const env = process.env.UPMARTO_URL?.trim();
    apiUrl = env ? env.replace(/\/$/, "") : await discoverBackend(workspace);
  }

  if (!(await probeBackend(apiUrl))) {
    throw new Error(
      `Upmarto backend not reachable at ${apiUrl}. ` +
        "Start the server (`cargo run`) then run: upmarto init",
    );
  }

  writeProjectConfig(workspace, {
    api_url: apiUrl,
    project_id: "auto",
    auto_capture: true,
    batch_size: 50,
    flush_interval_ms: 2000,
    retry_max: 5,
  });
  writeRuntimeFile(workspace, apiUrl, parsePort(apiUrl));
  process.env.UPMARTO_URL = apiUrl;
  return apiUrl;
}
