pub mod dto;
pub mod handlers;
pub mod routes;

use std::sync::Arc;

use crate::services::event_service::EventService;
use crate::services::explain_service::ExplainService;
use crate::services::session_service::SessionService;
use crate::services::timeline_service::TimelineService;
use crate::storage::repository::Repository;

/// Runtime values resolved after the HTTP listener binds (actual port, public URL).
#[derive(Debug, Clone)]
pub struct RuntimeInfo {
    pub api_base_url: String,
    pub host: String,
    pub port: u16,
}

#[derive(Clone)]
pub struct AppState {
    pub runtime: RuntimeInfo,
    pub repo: Arc<Repository>,
    pub event_service: Arc<EventService>,
    pub session_service: Arc<SessionService>,
    pub timeline_service: Arc<TimelineService>,
    pub explain_service: Arc<ExplainService>,
}

impl AppState {
    pub fn new(repo: Arc<Repository>, runtime: RuntimeInfo) -> Self {
        Self {
            runtime,
            event_service: Arc::new(EventService::new(repo.clone())),
            session_service: Arc::new(SessionService::new(repo.clone())),
            timeline_service: Arc::new(TimelineService::new(repo.clone())),
            explain_service: Arc::new(ExplainService::new(repo.clone())),
            repo,
        }
    }
}
