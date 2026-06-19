use std::sync::Arc;
use uuid::Uuid;

use crate::core::event::Event;
use crate::reasoning::{Explanation, WhyEngine};
use crate::storage::repository::Repository;

#[derive(Debug, thiserror::Error)]
pub enum ExplainError {
    #[error(transparent)]
    Storage(#[from] crate::storage::StorageError),
    #[error("event not found in session: {0}")]
    EventNotFound(String),
}

pub type ExplainResult<T> = Result<T, ExplainError>;

pub struct ExplainService {
    repo: Arc<Repository>,
}

impl ExplainService {
    pub fn new(repo: Arc<Repository>) -> Self {
        Self { repo }
    }

    pub fn explain(&self, session_id: &str, event_id: Option<&str>) -> ExplainResult<Explanation> {
        let events = self.repo.timeline_for_session(session_id)?;

        let scoped = match event_id {
            Some(id) => scope_to_event(events, id)?,
            None => events,
        };

        Ok(WhyEngine::explain(&scoped))
    }
}

fn scope_to_event(events: Vec<Event>, event_id: &str) -> ExplainResult<Vec<Event>> {
    let target =
        Uuid::parse_str(event_id).map_err(|_| ExplainError::EventNotFound(event_id.to_string()))?;

    let end = events
        .iter()
        .position(|e| e.id == target)
        .ok_or_else(|| ExplainError::EventNotFound(event_id.to_string()))?;

    Ok(events.into_iter().take(end + 1).collect())
}
