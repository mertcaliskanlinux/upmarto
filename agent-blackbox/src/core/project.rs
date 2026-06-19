use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct Project {
    pub id: String,
    pub created_at: i64,
    pub session_count: i64,
}
