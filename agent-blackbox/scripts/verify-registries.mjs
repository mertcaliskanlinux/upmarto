#!/usr/bin/env node
/**
 * Pre-publish registry onboarding checks for Upmarto.
 * Does NOT write credentials or tokens to the repository.
 *
 * Usage:
 *   node scripts/verify-registries.mjs
 *   node scripts/verify-registries.mjs --strict   # fail on warnings
 */
import { existsSync, readFileSync } from "node:fs";
import { homedir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { execSync, spawnSync } from "node:child_process";

const strict = process.argv.includes("--strict");
const root = join(dirname(fileURLToPath(import.meta.url)), "..");

const results = [];

function pass(name, detail) {
  results.push({ name, status: "pass", detail });
  console.log(`✓ ${name}: ${detail}`);
}

function warn(name, detail) {
  results.push({ name, status: "warn", detail });
  console.log(`⚠ ${name}: ${detail}`);
}

function fail(name, detail) {
  results.push({ name, status: "fail", detail });
  console.error(`✗ ${name}: ${detail}`);
}

function run(cmd, opts = {}) {
  return spawnSync(cmd, {
    shell: true,
    encoding: "utf8",
    stdio: ["ignore", "pipe", "pipe"],
    ...opts,
  });
}

function commandExists(cmd) {
  const probe = process.platform === "win32" ? `where ${cmd}` : `command -v ${cmd}`;
  return run(probe).status === 0;
}

// --- npm / @upmarto scope ---

function checkNpm() {
  if (!commandExists("npm")) {
    fail("npm CLI", "npm not found on PATH");
    return;
  }

  const whoami = run("npm whoami");
  if (whoami.status !== 0) {
    fail(
      "npm login",
      "not authenticated — run: npm login (use account with @upmarto org access)",
    );
    return;
  }

  const user = whoami.stdout.trim();
  pass("npm whoami", user);

  const registry = run("npm config get registry").stdout.trim();
  if (!registry.includes("registry.npmjs.org")) {
    warn("npm registry", `unexpected registry: ${registry}`);
  } else {
    pass("npm registry", registry);
  }

  const org = run("npm org ls upmarto");
  if (org.status === 0) {
    pass("@upmarto org", "membership confirmed via npm org ls upmarto");
  } else {
    warn(
      "@upmarto org",
      "could not list org (create org at npmjs.com or ensure membership). " +
        "Publish requires: npm publish --access public for @upmarto/sdk",
    );
  }

  const sdkView = run("npm view @upmarto/sdk version");
  if (sdkView.status === 0) {
    pass("@upmarto/sdk on npm", `published version ${sdkView.stdout.trim()}`);
  } else {
    warn("@upmarto/sdk on npm", "not published yet (expected before cursor/vscode registry deps)");
  }
}

// --- crates.io / cargo ---

function cargoCredentialsPresent() {
  const legacy = join(homedir(), ".cargo", "credentials");
  const toml = join(homedir(), ".cargo", "credentials.toml");
  if (existsSync(toml)) {
    const raw = readFileSync(toml, "utf8");
    if (raw.includes("crates.io") || raw.includes("[registry]")) {
      return true;
    }
  }
  if (existsSync(legacy)) {
    return true;
  }
  return false;
}

function checkCargo() {
  if (!commandExists("cargo")) {
    fail("cargo CLI", "cargo not found on PATH");
    return;
  }

  if (!cargoCredentialsPresent()) {
    fail(
      "cargo login",
      "no ~/.cargo/credentials.toml for crates.io — run: cargo login",
    );
    return;
  }
  pass("cargo login", "credentials file present (token not displayed)");

  for (const pkg of ["upmarto-sdk", "upmarto-cli"]) {
    const dry = run(`cargo publish -p ${pkg} --dry-run --allow-dirty`, {
      cwd: root,
      timeout: 120_000,
    });
    if (dry.status === 0) {
      pass(`cargo publish --dry-run (${pkg})`, "package validates");
    } else {
      const hint = (dry.stderr || dry.stdout || "").split("\n").slice(-3).join(" ").trim();
      fail(`cargo publish --dry-run (${pkg})`, hint || "dry-run failed");
    }
  }
}

// --- VS Code Marketplace / vsce ---

function checkVsce() {
  const ls = run("npx --yes @vscode/vsce ls-publishers", { timeout: 60_000 });
  if (ls.status !== 0) {
    fail(
      "vsce login",
      "could not list publishers — run: npx @vscode/vsce login upmarto",
    );
    return;
  }

  const publishers = ls.stdout
    .split(/\r?\n/)
    .map((l) => l.trim())
    .filter(Boolean);

  if (publishers.some((p) => p.toLowerCase() === "upmarto")) {
    pass("vsce publisher", 'publisher "upmarto" is available to this PAT');
  } else {
    fail(
      "vsce publisher",
      `publisher "upmarto" not in list [${publishers.join(", ")}]. ` +
        "Create publisher at marketplace.visualstudio.com",
    );
  }
}

// --- manifest files ---

function checkManifests() {
  const required = [
    "docs/PUBLISHING.md",
    "scripts/set-sdk-dependency.mjs",
    "upmarto-sdk-ts/package.json",
    "upmarto-sdk-rust/Cargo.toml",
    "upmarto-cli/Cargo.toml",
    "upmarto-vscode/package.json",
    "upmarto-vscode/media/icon-128.png",
  ];
  for (const rel of required) {
    const path = join(root, rel);
    if (existsSync(path)) {
      pass(`manifest ${rel}`, "present");
    } else {
      fail(`manifest ${rel}`, "missing");
    }
  }
}

console.log("Upmarto registry onboarding verification\n");
console.log(`Root: ${root}\n`);

checkManifests();
console.log("");
checkNpm();
console.log("");
checkCargo();
console.log("");
checkVsce();

const fails = results.filter((r) => r.status === "fail").length;
const warns = results.filter((r) => r.status === "warn").length;

console.log("\n--- Summary ---");
console.log(`Pass: ${results.filter((r) => r.status === "pass").length}`);
console.log(`Warn: ${warns}`);
console.log(`Fail: ${fails}`);

if (fails > 0 || (strict && warns > 0)) {
  console.error("\nRegistry onboarding NOT ready. See docs/REGISTRY_ONBOARDING.md");
  process.exit(1);
}

console.log("\nRegistry onboarding checks passed (warnings may remain until first publish).");
console.log("Next: docs/PUBLISHING.md publish order");
