#!/usr/bin/env node
/**
 * Switch @upmarto/sdk between monorepo file: link and registry semver.
 * Usage: node scripts/set-sdk-dependency.mjs local|registry
 */
import { readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const mode = process.argv[2];
if (mode !== "local" && mode !== "registry") {
  console.error("Usage: node scripts/set-sdk-dependency.mjs <local|registry>");
  process.exit(1);
}

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const consumers = ["upmarto-cursor/package.json", "upmarto-vscode/package.json"];
const value =
  mode === "local" ? "file:../upmarto-sdk-ts" : "^0.1.0";

for (const rel of consumers) {
  const path = join(root, rel);
  const pkg = JSON.parse(readFileSync(path, "utf8"));
  pkg.dependencies ??= {};
  pkg.dependencies["@upmarto/sdk"] = value;
  writeFileSync(path, `${JSON.stringify(pkg, null, 2)}\n`, "utf8");
  console.log(`${rel}: @upmarto/sdk -> ${value}`);
}
