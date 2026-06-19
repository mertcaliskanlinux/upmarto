import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { homedir } from "node:os";
import { join, resolve } from "node:path";

import type { UpmartoConfigFile } from "./types.js";

export function globalConfigPath(): string {
  return join(homedir(), ".upmarto", "config.json");
}

export function projectConfigPath(workspace: string): string {
  return join(resolve(workspace), ".upmarto", "config.json");
}

export function queuePath(workspace: string): string {
  return join(resolve(workspace), ".upmarto", "queue.jsonl");
}

export function loadMergedConfig(workspace: string): UpmartoConfigFile {
  const merged: UpmartoConfigFile = {
    api_url: "",
    project_id: "auto",
    auto_capture: true,
    batch_size: 50,
    flush_interval_ms: 2000,
    retry_max: 5,
  };

  const global = readConfig(globalConfigPath());
  const project = readConfig(projectConfigPath(workspace));
  if (global) Object.assign(merged, global);
  if (project) Object.assign(merged, project);

  const envUrl = process.env.UPMARTO_URL ?? process.env.BLACKBOX_URL;
  if (envUrl?.trim()) {
    merged.api_url = envUrl.trim().replace(/\/$/, "");
  }

  return merged;
}

function readConfig(path: string): UpmartoConfigFile | null {
  if (!existsSync(path)) return null;
  try {
    return JSON.parse(readFileSync(path, "utf8")) as UpmartoConfigFile;
  } catch {
    return null;
  }
}

export function writeProjectConfig(
  workspace: string,
  config: UpmartoConfigFile,
): void {
  const dir = join(resolve(workspace), ".upmarto");
  mkdirSync(dir, { recursive: true });
  writeFileSync(
    join(dir, "config.json"),
    JSON.stringify(config, null, 2),
    "utf8",
  );
}

export function hasProjectConfig(workspace: string): boolean {
  return existsSync(projectConfigPath(workspace));
}

export function deriveProjectId(workspace: string): string {
  const name = resolve(workspace).split(/[/\\]/).pop() ?? "workspace";
  return name.replace(/[^a-zA-Z0-9._-]/g, "-") || "workspace";
}

export function findProjectRoot(start = process.cwd()): string {
  let dir = resolve(start);
  while (true) {
    if (existsSync(join(dir, ".upmarto", "config.json"))) return dir;
    if (existsSync(join(dir, ".git"))) return dir;
    const parent = resolve(dir, "..");
    if (parent === dir) return start;
    dir = parent;
  }
}
