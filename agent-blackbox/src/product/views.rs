//! Product UX structure (no UI implementation).
//!
//! Defines how clients should conceptualize the three product surfaces.

use serde::{Deserialize, Serialize};

/// The **session** is the unit of analysis in Upmarto.
/// All events, timeline replay, and explanations are scoped to a session.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionView {
    pub session_id: String,
    pub project_id: String,
}

/// The **timeline** is the replay interface.
/// Presents the ordered stream of actions for session reconstruction and debugging.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TimelineView {
    pub session_id: String,
    /// Events are returned in strict timestamp order for deterministic replay.
    pub ordered: bool,
}

/// The **explain** view is the reasoning interface.
/// Answers "why did this happen?" from the event sequence alone.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExplainView {
    pub session_id: String,
    /// When set, reasoning is scoped to events up to and including this event.
    pub event_id: Option<String>,
}

/// Which product surface a client is interacting with.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProductSurface {
    /// POST /event — capture layer
    EventIngest,
    /// GET /timeline — replay layer
    Timeline,
    /// GET /session, GET /project/:id/sessions — session layer
    Session,
    /// POST /explain — reasoning layer
    Explain,
}

/// The canonical unit of analysis for agent behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnalysisUnit {
    Session,
}
