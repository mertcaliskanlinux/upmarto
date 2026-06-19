use std::sync::Arc;

use crate::core::event::Event;
use crate::storage::repository::Repository;

use super::ServiceResult;

pub struct TimelineService {
    repo: Arc<Repository>,
}

impl TimelineService {
    pub fn new(repo: Arc<Repository>) -> Self {
        Self { repo }
    }

    pub fn get_timeline(&self, session_id: &str) -> ServiceResult<Vec<Event>> {
        Ok(self.repo.timeline_for_session(session_id)?)
    }
}
