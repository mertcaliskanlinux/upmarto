import { appendFileSync, existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname } from "node:path";

import {
  deriveProjectId,
  findProjectRoot,
  loadMergedConfig,
  queuePath,
} from "./config.js";
import {
  NOT_CONFIGURED_MSG,
  validateWorkspaceAccess,
} from "./bootstrap.js";
import {
  dedupKey,
  DEDUP_WINDOW_MS,
  eventIdentityKey,
  parseQueueLine,
  toQueueLine,
} from "./queue_helpers.js";
import { resolveSessionId } from "./session.js";
import type {
  CreateEventRequest,
  ExplainResponseV1,
  TrackEvent,
  UpmartoInitOptions,
} from "./types.js";

type Queued = CreateEventRequest & { _attempts: number };

const DEFAULT_TIMEOUT_MS = 10_000;

function logStability(message: string): void {
  if (typeof process !== "undefined" && process.stderr?.write) {
    process.stderr.write(`[upmarto-sdk] ${message}\n`);
  }
}

export class Upmarto {
  private apiUrl: string;
  private projectId: string;
  private sessionId: string;
  private workspace: string;
  private queue: Queued[] = [];
  private flushTimer: ReturnType<typeof setInterval> | null = null;
  private flushing = false;
  private batchSize: number;
  private flushIntervalMs: number;
  private retryMax: number;
  private queueFile: string;
  private persistEnabled: boolean;

  /** In-memory dedup guard: dedupKey → last seen timestamp. */
  private readonly recentDedup = new Map<string, number>();
  private dedupSuppressedCount = 0;
  private retryRecoveryCount = 0;
  private rehydrationCount = 0;

  private constructor(opts: {
    apiUrl: string;
    projectId: string;
    sessionId: string;
    workspace: string;
    batchSize: number;
    flushIntervalMs: number;
    retryMax: number;
    persistEnabled: boolean;
  }) {
    this.apiUrl = opts.apiUrl;
    this.projectId = opts.projectId;
    this.sessionId = opts.sessionId;
    this.workspace = opts.workspace;
    this.batchSize = opts.batchSize;
    this.flushIntervalMs = opts.flushIntervalMs;
    this.retryMax = opts.retryMax;
    this.persistEnabled = opts.persistEnabled;
    this.queueFile = opts.persistEnabled ? queuePath(opts.workspace) : "";
    if (this.persistEnabled) this.restorePersistedQueue();
  }

  /** Initialize SDK — works in Node.js and browser (browser: no disk queue). */
  static init(options: UpmartoInitOptions): Upmarto {
    const persistEnabled =
      typeof process !== "undefined" && Boolean(process.versions?.node);
    const workspace = options.workspacePath ?? (persistEnabled ? findProjectRoot() : ".");
    const apiUrl = options.apiUrl.replace(/\/$/, "");
    const projectId =
      options.projectId && options.projectId !== "auto"
        ? options.projectId
        : deriveProjectId(workspace);

    const client = new Upmarto({
      apiUrl,
      projectId,
      sessionId: resolveSessionId(workspace),
      workspace,
      batchSize: options.batchSize ?? 50,
      flushIntervalMs: options.flushIntervalMs ?? 2000,
      retryMax: options.retryMax ?? 5,
      persistEnabled,
    });

    if (options.autoFlush !== false) {
      client.startAutoFlush();
    }

    return client;
  }

  /** Load from `.upmarto/config.json` + env (Node.js). Validates backend unless `UPMARTO_URL` is set. */
  static async fromWorkspace(workspace?: string): Promise<Upmarto> {
    const root = workspace ?? findProjectRoot();
    const cfg = loadMergedConfig(root);
    if (!cfg.api_url) {
      logStability(NOT_CONFIGURED_MSG);
      throw new Error(NOT_CONFIGURED_MSG);
    }

    try {
      await validateWorkspaceAccess(root, cfg.api_url);
    } catch (err) {
      const message = err instanceof Error ? err.message : NOT_CONFIGURED_MSG;
      logStability(message);
      throw err;
    }

    return Upmarto.init({
      apiUrl: cfg.api_url,
      projectId: cfg.project_id ?? "auto",
      workspacePath: root,
      batchSize: cfg.batch_size,
      flushIntervalMs: cfg.flush_interval_ms,
      retryMax: cfg.retry_max,
    });
  }

  session(id: string): void {
    this.sessionId = id;
  }

  getSessionId(): string {
    return this.sessionId;
  }

  getProjectId(): string {
    return this.projectId;
  }

  track(event: TrackEvent): void {
    const req: Queued = {
      project_id: this.projectId,
      session_id: this.sessionId,
      event_type: event.event_type,
      timestamp: event.timestamp ?? Date.now(),
      payload: event.payload,
      _attempts: 0,
    };

    if (this.isDuplicate(req)) {
      this.dedupSuppressedCount += 1;
      logStability(
        `duplicate suppressed (total=${this.dedupSuppressedCount}): ${req.event_type}`,
      );
      return;
    }

    if (this.persistEnabled) this.persistEvent(req);
    this.queue.push(req);

    if (this.queue.length >= this.batchSize) {
      void this.flush();
    }
  }

  async flush(): Promise<void> {
    if (this.flushing || this.queue.length === 0) return;
    this.flushing = true;

    try {
      while (this.queue.length > 0) {
        const flushed = await this.flushBatch(
          this.queue.splice(0, this.batchSize),
        );
        if (!flushed) {
          break;
        }
      }
    } finally {
      this.flushing = false;
    }
  }

  async explain(sessionId?: string): Promise<ExplainResponseV1> {
    const sid = sessionId ?? this.sessionId;
    const res = await this.fetchWithTimeout(`${this.apiUrl}/explain`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ session_id: sid }),
    });
    if (!res.ok) {
      const body = await res.text().catch(() => "");
      throw new Error(`POST /explain ${res.status}: ${body.slice(0, 200)}`);
    }
    return res.json() as Promise<ExplainResponseV1>;
  }

  private isDuplicate(req: CreateEventRequest): boolean {
    const now = req.timestamp ?? Date.now();
    const key = dedupKey(req);
    const last = this.recentDedup.get(key);
    if (last !== undefined && now - last < DEDUP_WINDOW_MS) {
      return true;
    }
    this.recentDedup.set(key, now);

    if (this.recentDedup.size > 512) {
      const cutoff = now - DEDUP_WINDOW_MS * 2;
      for (const [k, ts] of this.recentDedup) {
        if (ts < cutoff) this.recentDedup.delete(k);
      }
    }
    return false;
  }

  /** Flush one batch with bounded retries; never drops events. */
  private async flushBatch(batch: Queued[]): Promise<boolean> {
    const maxAttempts = Math.max(this.retryMax, 1);
    let pending = batch;

    for (let attempt = 0; attempt < maxAttempts; attempt++) {
      const failed: Queued[] = [];

      for (const item of pending) {
        try {
          await this.postEvent(item);
          if (this.persistEnabled) this.removePersisted(item);
        } catch {
          item._attempts += 1;
          failed.push(item);
        }
      }

      if (failed.length === 0) {
        return true;
      }

      if (attempt + 1 < maxAttempts) {
        const delay = 250 * 2 ** attempt;
        logStability(
          `flush retry ${attempt + 1}/${maxAttempts} (${failed.length} pending)`,
        );
        await new Promise((r) => setTimeout(r, delay));
        pending = failed;
        continue;
      }

      for (const item of failed.reverse()) {
        this.queue.unshift(item);
        if (item._attempts >= this.retryMax) {
          this.retryRecoveryCount += 1;
        }
      }
      logStability(
        `retry recovery re-queued ${failed.length} event(s) (total recoveries=${this.retryRecoveryCount})`,
      );
      return false;
    }

    return true;
  }

  private startAutoFlush(): void {
    if (this.flushTimer) return;
    this.flushTimer = setInterval(() => {
      void this.flush();
    }, this.flushIntervalMs);
    if (typeof this.flushTimer === "object" && "unref" in this.flushTimer) {
      this.flushTimer.unref();
    }
  }

  private async postEvent(event: CreateEventRequest): Promise<void> {
    const res = await this.fetchWithTimeout(`${this.apiUrl}/event`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        project_id: event.project_id,
        session_id: event.session_id,
        event_type: event.event_type,
        timestamp: event.timestamp,
        payload: event.payload,
      }),
    });
    if (!res.ok) {
      const body = await res.text().catch(() => "");
      throw new Error(`POST /event ${res.status}: ${body.slice(0, 200)}`);
    }
  }

  private fetchWithTimeout(url: string, init: RequestInit): Promise<Response> {
    const controller = new AbortController();
    const timer = setTimeout(() => controller.abort(), DEFAULT_TIMEOUT_MS);
    return fetch(url, { ...init, signal: controller.signal }).finally(() =>
      clearTimeout(timer),
    );
  }

  private persistEvent(event: CreateEventRequest): void {
    try {
      const dir = dirname(this.queueFile);
      if (!existsSync(dir)) mkdirSync(dir, { recursive: true });
      appendFileSync(this.queueFile, `${toQueueLine(event)}\n`, "utf8");
    } catch {
      // offline tolerance
    }
  }

  private removePersisted(sent: CreateEventRequest): void {
    try {
      if (!existsSync(this.queueFile)) return;
      const targetKey = eventIdentityKey(sent);
      let removed = false;
      const kept: string[] = [];

      for (const line of readFileSync(this.queueFile, "utf8").split("\n")) {
        if (!line.trim()) continue;
        if (!removed) {
          const parsed = parseQueueLine(line);
          if (parsed && eventIdentityKey(parsed) === targetKey) {
            removed = true;
            continue;
          }
        }
        kept.push(line);
      }

      if (linesEmpty(kept)) {
        writeFileSync(this.queueFile, "", "utf8");
      } else {
        writeFileSync(this.queueFile, `${kept.join("\n")}\n`, "utf8");
      }
    } catch {
      // ignore
    }
  }

  private restorePersistedQueue(): void {
    try {
      if (!existsSync(this.queueFile)) return;
      for (const line of readFileSync(this.queueFile, "utf8").split("\n")) {
        if (!line.trim()) continue;
        const req = parseQueueLine(line);
        if (!req) continue;
        this.queue.push({ ...req, _attempts: 0 });
        this.rehydrationCount += 1;
      }
      if (this.rehydrationCount > 0) {
        logStability(`queue rehydrated ${this.rehydrationCount} event(s) from disk`);
      }
    } catch {
      // ignore
    }
  }
}

function linesEmpty(lines: string[]): boolean {
  return lines.length === 0 || lines.every((l) => !l.trim());
}
