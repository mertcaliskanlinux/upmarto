import { getApiBase } from "./config";
import type {
  ApiErrorV1,
  ExplainRequestV1,
  ExplainResponseV1,
  SessionsResponseV1,
  TimelineResponseV1,
} from "./types";

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const base = await getApiBase();
  const res = await fetch(`${base}${path}`, {
    ...init,
    headers: {
      "Content-Type": "application/json",
      ...init?.headers,
    },
  });

  if (!res.ok) {
    let message = res.statusText;
    try {
      const body = (await res.json()) as ApiErrorV1;
      message = body.error ?? message;
    } catch {
      /* ignore */
    }
    throw new Error(message);
  }

  return res.json() as Promise<T>;
}

export function fetchProjectSessions(projectId: string): Promise<SessionsResponseV1> {
  return request(`/project/${encodeURIComponent(projectId)}/sessions`);
}

export function fetchProjects(): Promise<{ projects: string[] }> {
  return request("/debug/projects");
}

export function fetchTimeline(sessionId: string): Promise<TimelineResponseV1> {
  return request(`/timeline?session_id=${encodeURIComponent(sessionId)}`);
}

export function fetchExplain(body: ExplainRequestV1): Promise<ExplainResponseV1> {
  return request("/explain", {
    method: "POST",
    body: JSON.stringify(body),
  });
}

export function fetchRuntimeConfig() {
  return request<import("./config").RuntimeConfigV1>("/config");
}
