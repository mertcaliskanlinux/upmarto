import { createHash } from "node:crypto";

import type { CreateEventRequest } from "./types.js";

/** Semantic fields only — never persist `_attempts` or other internal metadata. */
export function toQueueLine(event: CreateEventRequest): string {
  return JSON.stringify({
    project_id: event.project_id,
    session_id: event.session_id,
    event_type: event.event_type,
    timestamp: event.timestamp,
    payload: event.payload,
  });
}

/** Parse a queue line, stripping legacy `_attempts` and other non-contract fields. */
export function parseQueueLine(line: string): CreateEventRequest | null {
  try {
    const raw = JSON.parse(line) as Record<string, unknown>;
    if (
      typeof raw.project_id !== "string" ||
      typeof raw.session_id !== "string" ||
      typeof raw.event_type !== "string" ||
      typeof raw.payload !== "object" ||
      raw.payload === null
    ) {
      return null;
    }
    const req: CreateEventRequest = {
      project_id: raw.project_id,
      session_id: raw.session_id,
      event_type: raw.event_type as CreateEventRequest["event_type"],
      payload: raw.payload as Record<string, unknown>,
    };
    if (typeof raw.timestamp === "number") {
      req.timestamp = raw.timestamp;
    }
    return req;
  } catch {
    return null;
  }
}

export function stableStringify(value: unknown): string {
  if (value === null || typeof value !== "object") {
    return JSON.stringify(value);
  }
  if (Array.isArray(value)) {
    return `[${value.map(stableStringify).join(",")}]`;
  }
  const obj = value as Record<string, unknown>;
  const keys = Object.keys(obj).sort();
  return `{${keys.map((k) => `${JSON.stringify(k)}:${stableStringify(obj[k])}`).join(",")}}`;
}

export function payloadHash(payload: Record<string, unknown>): string {
  return createHash("sha256").update(stableStringify(payload)).digest("hex").slice(0, 16);
}

/** Identity for queue removal and deduplication (semantic fields only). */
export function eventIdentityKey(event: CreateEventRequest): string {
  const ts = event.timestamp ?? 0;
  return `${event.session_id}|${event.event_type}|${ts}|${payloadHash(event.payload)}`;
}

export function dedupKey(event: CreateEventRequest): string {
  return `${event.session_id}|${event.event_type}|${payloadHash(event.payload)}`;
}

export const DEDUP_WINDOW_MS = 500;
