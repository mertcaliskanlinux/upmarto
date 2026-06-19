use std::sync::Arc;

use crate::core::session::Session;
use crate::storage::repository::Repository;

use super::ServiceResult;

pub struct SessionService {
    repo: Arc<Repository>,
}

impl SessionService {
    pub fn new(repo: Arc<Repository>) -> Self {
        Self { repo }
    }

    pub fn get_session(&self, session_id: &str) -> ServiceResult<Session> {
        Ok(self.repo.get_session(session_id)?)
    }

    pub fn list_project_sessions(&self, project_id: &str) -> ServiceResult<Vec<Session>> {
        Ok(self.repo.list_project_sessions(project_id)?)
    }
}
