use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Frozen v1 event types — must match backend `product/contract.rs`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    FileOpened,
    FileModified,
    FileCreated,
    CommandExecuted,
    TestRun,
    TestFailed,
    TestPassed,
    GitCommit,
    AgentMessage,
}

impl EventType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::FileOpened => "file_opened",
            Self::FileModified => "file_modified",
            Self::FileCreated => "file_created",
            Self::CommandExecuted => "command_executed",
            Self::TestRun => "test_run",
            Self::TestFailed => "test_failed",
            Self::TestPassed => "test_passed",
            Self::GitCommit => "git_commit",
            Self::AgentMessage => "agent_message",
        }
    }
}

/// Event payload for `Upmarto::track`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackEvent {
    pub event_type: EventType,
    pub payload: Value,
    pub timestamp: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEventRequest {
    pub project_id: String,
    pub session_id: String,
    pub event_type: EventType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<i64>,
    pub payload: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplainRequest {
    pub session_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ExplainResponse {
    pub api_version: String,
    pub explain_schema_version: String,
    pub summary: String,
    pub root_cause: String,
    pub decision_chain: Vec<String>,
    pub problem_statement: String,
    pub resolution_flow: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TimelineResponse {
    pub api_version: String,
    pub session_id: String,
    pub events: Vec<serde_json::Value>,
}
