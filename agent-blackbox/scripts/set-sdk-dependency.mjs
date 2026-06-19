#!/usr/bin/env node
/**
 * Switch @upmarto/sdk between monorepo file: link and registry semver.
 *
 * Usage:
 *   node scripts/set-sdk-dependency.mjs local
 *   node scripts/set-sdk-dependency.mjs registry
 *   node scripts/set-sdk-dependency.mjs registry --verify
 *
 * --verify (registry only): pack local SDK tarball, install into consumers,
 * run compile/build, then leave registry semver in package.json.
 * Reverts to file: on failure. Never writes credentials.
 */
import {
  existsSync,
  readFileSync,
  readdirSync,
  unlinkSync,
  writeFileSync,
} from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { spawnSync } from "node:child_process";

const mode = process.argv[2];
const verify = process.argv.includes("--verify");

if (mode !== "local" && mode !== "registry") {
  console.error(
    "Usage: node scripts/set-sdk-dependency.mjs <local|registry> [--verify]",
  );
  process.exit(1);
}

if (verify && mode !== "registry") {
  console.error("--verify is only valid with registry mode");
  process.exit(1);
}

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const consumers = [
  { rel: "upmarto-cursor/package.json", script: "build" },
  { rel: "upmarto-vscode/package.json", script: "compile" },
];
const sdkDir = join(root, "upmarto-sdk-ts");
const registryVersion = "^0.1.0";
const localPath = "file:../upmarto-sdk-ts";

function readPkg(rel) {
  return JSON.parse(readFileSync(join(root, rel), "utf8"));
}

function writePkg(rel, pkg) {
  writeFileSync(join(root, rel), `${JSON.stringify(pkg, null, 2)}\n`, "utf8");
}

function setDependency(value) {
  for (const { rel } of consumers) {
    const pkg = readPkg(rel);
    pkg.dependencies ??= {};
    pkg.dependencies["@upmarto/sdk"] = value;
    writePkg(rel, pkg);
    console.log(`${rel}: @upmarto/sdk -> ${value}`);
  }
}

function run(cmd, cwd) {
  const result = spawnSync(cmd, {
    shell: true,
    cwd,
    encoding: "utf8",
    stdio: "inherit",
  });
  if (result.status !== 0) {
    throw new Error(`Command failed (${result.status}): ${cmd}`);
  }
}

function findPackedSdkTarball() {
  const files = readdirSync(sdkDir).filter(
    (f) => f.startsWith("upmarto-sdk-") && f.endsWith(".tgz"),
  );
  if (files.length === 0) {
    throw new Error("npm pack did not produce upmarto-sdk-*.tgz");
  }
  files.sort();
  return join(sdkDir, files[files.length - 1]);
}

function cleanupPackedTarballs() {
  for (const f of readdirSync(sdkDir)) {
    if (f.startsWith("upmarto-sdk-") && f.endsWith(".tgz")) {
      unlinkSync(join(sdkDir, f));
    }
  }
}

function verifyRegistryCompile() {
  console.log("\n--- Registry swap verify (local tarball stand-in) ---\n");

  setDependency(registryVersion);

  console.log("Packing @upmarto/sdk...");
  cleanupPackedTarballs();
  run("npm run build", sdkDir);
  run("npm pack --silent", sdkDir);
  const tarball = findPackedSdkTarball();
  console.log(`Using tarball: ${tarball}`);

  const tarballDep = `file:${tarball.replace(/\\/g, "/")}`;

  for (const { rel, script } of consumers) {
    const pkg = readPkg(rel);
    pkg.dependencies["@upmarto/sdk"] = tarballDep;
    writePkg(rel, pkg);

    const dir = dirname(join(root, rel));
    console.log(`\nVerifying ${rel}...`);
    run("npm install --no-audit --no-fund", dir);
    run(`npm run ${script}`, dir);
    console.log(`✓ ${rel} ${script} OK`);

    pkg.dependencies["@upmarto/sdk"] = registryVersion;
    writePkg(rel, pkg);
  }

  cleanupPackedTarballs();
  console.log("\n--- Registry verify passed; package.json set to ^0.1.0 ---\n");
}

try {
  if (mode === "local") {
    setDependency(localPath);
    process.exit(0);
  }

  if (verify) {
    verifyRegistryCompile();
  } else {
    setDependency(registryVersion);
    console.log(
      "\nTip: run with --verify to compile consumers against a packed SDK tarball before publish.",
    );
  }
} catch (err) {
  console.error(`\nError: ${err.message}`);
  console.error("Reverting to local file: dependencies...");
  try {
    setDependency(localPath);
  } catch {
    console.error("Could not revert package.json — fix manually.");
  }
  try {
    cleanupPackedTarballs();
  } catch {
    /* ignore */
  }
  process.exit(1);
}
