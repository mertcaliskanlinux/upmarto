//! Frozen public API contract (v1).
//!
//! All request/response types in this module are the stable product surface.
//! Breaking changes require a new version constant and parallel types (v2).

use serde::{Deserialize, Serialize};

use crate::core::event::{CreateEventRequest, Event};
use crate::core::session::Session;
use crate::reasoning::Explanation;
use crate::replay::replay_engine::ReplaySummary;

// ---------------------------------------------------------------------------
// Version constants — bump only on breaking contract changes
// ---------------------------------------------------------------------------

pub const API_VERSION: &str = "v1";
pub const EVENT_SCHEMA_VERSION: &str = "v1";
pub const EXPLAIN_SCHEMA_VERSION: &str = "v1";
pub const PRODUCT_NAME: &str = "Upmarto";
pub const PRODUCT_TAGLINE: &str = "Memory and Reasoning for AI Agents";

/// Frozen event type identifiers. No additions without EVENT_SCHEMA_VERSION bump.
pub const EVENT_TYPES_V1: &[&str] = &[
    "file_opened",
    "file_modified",
    "file_created",
    "command_executed",
    "test_run",
    "test_failed",
    "test_passed",
    "git_commit",
    "agent_message",
];

// ---------------------------------------------------------------------------
// Event layer — POST /event
// ---------------------------------------------------------------------------

/// Public ingest request. Alias of internal type; validated at API boundary.
pub type CreateEventRequestV1 = CreateEventRequest;

#[derive(Debug, Serialize)]
pub struct EventResponseV1 {
    pub api_version: &'static str,
    pub event_schema_version: &'static str,
    pub event: Event,
}

impl EventResponseV1 {
    pub fn new(event: Event) -> Self {
        Self {
            api_version: API_VERSION,
            event_schema_version: EVENT_SCHEMA_VERSION,
            event,
        }
    }
}

pub fn event_response(event: Event) -> EventResponseV1 {
    EventResponseV1::new(event)
}

// ---------------------------------------------------------------------------
// Timeline layer — GET /timeline
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct TimelineQueryV1 {
    pub session_id: String,
}

#[derive(Debug, Serialize)]
pub struct TimelineResponseV1 {
    pub api_version: &'static str,
    pub session_id: String,
    pub events: Vec<Event>,
    pub summary: ReplaySummary,
}

impl TimelineResponseV1 {
    pub fn new(session_id: String, events: Vec<Event>, summary: ReplaySummary) -> Self {
        Self {
            api_version: API_VERSION,
            session_id,
            events,
            summary,
        }
    }
}

pub fn timeline_response(
    session_id: String,
    events: Vec<Event>,
    summary: ReplaySummary,
) -> TimelineResponseV1 {
    TimelineResponseV1::new(session_id, events, summary)
}

// ---------------------------------------------------------------------------
// Session layer — GET /session/:id, GET /project/:id/sessions
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct SessionResponseV1 {
    pub api_version: &'static str,
    pub session: Session,
}

impl SessionResponseV1 {
    pub fn new(session: Session) -> Self {
        Self {
            api_version: API_VERSION,
            session,
        }
    }
}

pub fn session_response(session: Session) -> SessionResponseV1 {
    SessionResponseV1::new(session)
}

#[derive(Debug, Serialize)]
pub struct SessionsResponseV1 {
    pub api_version: &'static str,
    pub project_id: String,
    pub sessions: Vec<Session>,
}

impl SessionsResponseV1 {
    pub fn new(project_id: String, sessions: Vec<Session>) -> Self {
        Self {
            api_version: API_VERSION,
            project_id,
            sessions,
        }
    }
}

pub fn sessions_response(project_id: String, sessions: Vec<Session>) -> SessionsResponseV1 {
    SessionsResponseV1::new(project_id, sessions)
}

// ---------------------------------------------------------------------------
// Reasoning layer — POST /explain (CORE PRODUCT FEATURE)
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct ExplainRequestV1 {
    pub session_id: String,
    pub event_id: Option<String>,
}

/// Frozen explain output schema. Field set is immutable for v1.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExplainResponseV1 {
    pub api_version: &'static str,
    pub explain_schema_version: &'static str,
    pub summary: String,
    pub root_cause: String,
    pub decision_chain: Vec<String>,
    pub problem_statement: String,
    pub resolution_flow: String,
}

impl From<Explanation> for ExplainResponseV1 {
    fn from(explanation: Explanation) -> Self {
        Self {
            api_version: API_VERSION,
            explain_schema_version: EXPLAIN_SCHEMA_VERSION,
            summary: explanation.summary,
            root_cause: explanation.root_cause,
            decision_chain: explanation.decision_chain,
            problem_statement: explanation.problem_statement,
            resolution_flow: explanation.resolution_flow,
        }
    }
}

// ---------------------------------------------------------------------------
// Runtime config — GET /config
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct ConfigResponseV1 {
    pub api_version: &'static str,
    pub product_name: &'static str,
    pub product_tagline: &'static str,
    pub api_base_url: String,
    pub host: String,
    pub port: u16,
}

impl ConfigResponseV1 {
    pub fn new(api_base_url: String, host: String, port: u16) -> Self {
        Self {
            api_version: API_VERSION,
            product_name: PRODUCT_NAME,
            product_tagline: PRODUCT_TAGLINE,
            api_base_url,
            host,
            port,
        }
    }
}

pub fn config_response(api_base_url: String, host: String, port: u16) -> ConfigResponseV1 {
    ConfigResponseV1::new(api_base_url, host, port)
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct ErrorResponseV1 {
    pub api_version: &'static str,
    pub error: String,
}

impl ErrorResponseV1 {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            api_version: API_VERSION,
            error: message.into(),
        }
    }
}

pub fn error_response(message: impl Into<String>) -> ErrorResponseV1 {
    ErrorResponseV1::new(message)
}

pub type ApiErrorBody = ErrorResponseV1;

// ---------------------------------------------------------------------------
// Validation
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    pub field: &'static str,
    pub message: String,
}

impl ValidationError {
    pub fn new(field: &'static str, message: impl Into<String>) -> Self {
        Self {
            field,
            message: message.into(),
        }
    }
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.field, self.message)
    }
}

/// Strict schema validation for event ingestion. Rejects malformed requests
/// before they reach storage.
pub fn validate_create_event(request: &CreateEventRequestV1) -> Result<(), ValidationError> {
    if request.project_id.trim().is_empty() {
        return Err(ValidationError::new("project_id", "must not be empty"));
    }

    if request.session_id.trim().is_empty() {
        return Err(ValidationError::new("session_id", "must not be empty"));
    }

    if !request.payload.is_object() {
        return Err(ValidationError::new("payload", "must be a JSON object"));
    }

    if let Some(ts) = request.timestamp {
        if ts < 0 {
            return Err(ValidationError::new(
                "timestamp",
                "must be a non-negative millisecond value",
            ));
        }
    }

    Ok(())
}

pub fn validate_explain_request(request: &ExplainRequestV1) -> Result<(), ValidationError> {
    if request.session_id.trim().is_empty() {
        return Err(ValidationError::new("session_id", "must not be empty"));
    }

    if let Some(event_id) = &request.event_id {
        if event_id.trim().is_empty() {
            return Err(ValidationError::new(
                "event_id",
                "must not be empty when provided",
            ));
        }
        if uuid::Uuid::parse_str(event_id).is_err() {
            return Err(ValidationError::new("event_id", "must be a valid UUID"));
        }
    }

    Ok(())
}

pub fn validate_timeline_query(query: &TimelineQueryV1) -> Result<(), ValidationError> {
    if query.session_id.trim().is_empty() {
        return Err(ValidationError::new("session_id", "must not be empty"));
    }
    Ok(())
}

pub fn validate_path_param(field: &'static str, value: &str) -> Result<(), ValidationError> {
    if value.trim().is_empty() {
        return Err(ValidationError::new(field, "must not be empty"));
    }
    Ok(())
}
