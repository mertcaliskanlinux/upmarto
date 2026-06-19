use std::sync::Arc;
use uuid::Uuid;

use crate::core::event::{CreateEventRequest, Event};
use crate::storage::repository::Repository;
use crate::utils::time::now_millis;

use super::ServiceResult;

pub struct EventService {
    repo: Arc<Repository>,
}

impl EventService {
    pub fn new(repo: Arc<Repository>) -> Self {
        Self { repo }
    }

    pub fn ingest(&self, request: CreateEventRequest) -> ServiceResult<Event> {
        let timestamp = request.timestamp.unwrap_or_else(now_millis);
        let event = request.into_event(Uuid::new_v4(), timestamp);
        self.repo.store_event(&event)?;
        Ok(event)
    }
}
