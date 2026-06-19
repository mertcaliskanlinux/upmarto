use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use super::types::EventType;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: Uuid,
    pub timestamp: i64,
    pub project_id: String,
    pub session_id: String,
    pub event_type: EventType,
    pub payload: Value,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateEventRequest {
    pub timestamp: Option<i64>,
    pub project_id: String,
    pub session_id: String,
    pub event_type: EventType,
    pub payload: Value,
}

impl CreateEventRequest {
    pub fn into_event(self, id: Uuid, timestamp: i64) -> Event {
        Event {
            id,
            timestamp,
            project_id: self.project_id,
            session_id: self.session_id,
            event_type: self.event_type,
            payload: self.payload,
        }
    }
}
