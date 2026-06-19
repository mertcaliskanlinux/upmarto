/** Frozen v1 API types — must match backend product/contract.rs */

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

export interface Event {
  id: string;
  timestamp: number;
  project_id: string;
  session_id: string;
  event_type: EventType;
  payload: Record<string, unknown>;
}

export interface Session {
  id: string;
  project_id: string;
  started_at: number;
  ended_at: number | null;
  event_count: number;
}

export interface ReplaySummary {
  total_events: number;
  file_events: number;
  command_events: number;
  test_events: number;
  git_events: number;
  agent_messages: number;
}

export interface SessionsResponseV1 {
  api_version: string;
  project_id: string;
  sessions: Session[];
}

export interface TimelineResponseV1 {
  api_version: string;
  session_id: string;
  events: Event[];
  summary: ReplaySummary;
}

export interface ExplainRequestV1 {
  session_id: string;
  event_id?: string;
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

export interface ApiErrorV1 {
  api_version: string;
  error: string;
}
