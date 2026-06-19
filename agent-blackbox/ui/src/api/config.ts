/** Runtime API base URL resolution — no hardcoded hosts or ports. */

export interface RuntimeConfigV1 {
  api_version: string;
  product_name: string;
  product_tagline: string;
  api_base_url: string;
  host: string;
  port: number;
}

let cachedBase: string | null = null;
let resolvePromise: Promise<string> | null = null;

function trimBase(url: string): string {
  return url.replace(/\/$/, "");
}

/** Explicit override from build-time env (VITE_API_BASE_URL). */
export function getEnvApiBase(): string {
  const raw = import.meta.env.VITE_API_BASE_URL?.trim();
  return raw ? trimBase(raw) : "";
}

async function discoverApiBase(): Promise<string> {
  const envBase = getEnvApiBase();
  if (envBase) return envBase;

  const origin =
    typeof window !== "undefined" && window.location?.origin
      ? window.location.origin
      : "";

  if (origin) {
    try {
      const res = await fetch(`${origin}/config`, { headers: { Accept: "application/json" } });
      if (res.ok) {
        const body = (await res.json()) as RuntimeConfigV1;
        if (body.api_base_url) return trimBase(body.api_base_url);
      }
    } catch {
      /* same-origin proxy or static hosting — fall through */
    }
    return origin;
  }

  return "";
}

/** Cached async resolver — prefers env, then GET /config, then window.location.origin. */
export function getApiBase(): Promise<string> {
  if (cachedBase !== null) return Promise.resolve(cachedBase);
  if (!resolvePromise) {
    resolvePromise = discoverApiBase().then((base) => {
      cachedBase = base;
      return base;
    });
  }
  return resolvePromise;
}

/** Synchronous base when already resolved (empty string = relative same-origin). */
export function getApiBaseSync(): string {
  return cachedBase ?? getEnvApiBase();
}
