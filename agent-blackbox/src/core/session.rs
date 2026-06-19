use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Session {
    pub id: String,
    pub project_id: String,
    pub started_at: i64,
    pub ended_at: Option<i64>,
    pub event_count: i64,
}
