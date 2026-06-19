/** Frozen v1 event types — must match backend contract. */

export type EventType =
  | "file_opened"
  | "file_modified"
  | "file_created"
  | "command_executed"
  | "test_run"
  | "test_failed"
  | "test_passed"
  | "git_commit"
  | "agent_message";

export interface TrackEvent {
  event_type: EventType;
  payload: Record<string, unknown>;
  timestamp?: number;
}

export interface CreateEventRequest {
  project_id: string;
  session_id: string;
  event_type: EventType;
  timestamp?: number;
  payload: Record<string, unknown>;
}

export interface ExplainResponseV1 {
  api_version: string;
  explain_schema_version: string;
  summary: string;
  root_cause: string;
  decision_chain: string[];
  problem_statement: string;
  resolution_flow: string;
}

export interface UpmartoInitOptions {
  apiUrl: string;
  projectId?: string;
  workspacePath?: string;
  batchSize?: number;
  flushIntervalMs?: number;
  retryMax?: number;
  autoFlush?: boolean;
}

export interface UpmartoConfigFile {
  api_url: string;
  project_id?: string;
  auto_capture?: boolean;
  batch_size?: number;
  flush_interval_ms?: number;
  retry_max?: number;
}

export const EVENT_TYPES_V1: readonly EventType[] = [
  "file_opened",
  "file_modified",
  "file_created",
  "command_executed",
  "test_run",
  "test_failed",
  "test_passed",
  "git_commit",
  "agent_message",
] as const;
