use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};

use crate::core::event::CreateEventRequest;
use crate::product::contract::{
    config_response, error_response, event_response, session_response, sessions_response,
    timeline_response, validate_create_event, validate_explain_request, validate_path_param,
    validate_timeline_query, ConfigResponseV1, ErrorResponseV1, EventResponseV1, ExplainRequestV1,
    ExplainResponseV1, SessionResponseV1, SessionsResponseV1, TimelineQueryV1, TimelineResponseV1,
    ValidationError,
};
use crate::replay::replay_engine::ReplayEngine;
use crate::services::explain_service::ExplainError;
use crate::services::ServiceError;
use crate::storage::StorageError;

use super::AppState;

pub async fn get_config(State(state): State<AppState>) -> Json<ConfigResponseV1> {
    Json(config_response(
        state.runtime.api_base_url.clone(),
        state.runtime.host.clone(),
        state.runtime.port,
    ))
}

pub async fn post_event(
    State(state): State<AppState>,
    Json(request): Json<CreateEventRequest>,
) -> Result<(StatusCode, Json<EventResponseV1>), (StatusCode, Json<ErrorResponseV1>)> {
    if let Err(err) = validate_create_event(&request) {
        return Err(map_validation_error(err));
    }

    match state.event_service.ingest(request) {
        Ok(event) => Ok((StatusCode::CREATED, Json(event_response(event)))),
        Err(err) => Err(map_error(err)),
    }
}

pub async fn get_timeline(
    State(state): State<AppState>,
    Query(query): Query<TimelineQueryV1>,
) -> Result<Json<TimelineResponseV1>, (StatusCode, Json<ErrorResponseV1>)> {
    if let Err(err) = validate_timeline_query(&query) {
        return Err(map_validation_error(err));
    }

    match state.timeline_service.get_timeline(&query.session_id) {
        Ok(events) => {
            let summary = ReplayEngine::summarize(&events);
            Ok(Json(timeline_response(query.session_id, events, summary)))
        }
        Err(err) => Err(map_error(err)),
    }
}

pub async fn get_session(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> Result<Json<SessionResponseV1>, (StatusCode, Json<ErrorResponseV1>)> {
    if let Err(err) = validate_path_param("session_id", &session_id) {
        return Err(map_validation_error(err));
    }

    match state.session_service.get_session(&session_id) {
        Ok(session) => Ok(Json(session_response(session))),
        Err(err) => Err(map_error(err)),
    }
}

pub async fn get_project_sessions(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
) -> Result<Json<SessionsResponseV1>, (StatusCode, Json<ErrorResponseV1>)> {
    if let Err(err) = validate_path_param("project_id", &project_id) {
        return Err(map_validation_error(err));
    }

    match state.session_service.list_project_sessions(&project_id) {
        Ok(sessions) => Ok(Json(sessions_response(project_id, sessions))),
        Err(err) => Err(map_error(err)),
    }
}

pub async fn post_explain(
    State(state): State<AppState>,
    Json(request): Json<ExplainRequestV1>,
) -> Result<Json<ExplainResponseV1>, (StatusCode, Json<ErrorResponseV1>)> {
    if let Err(err) = validate_explain_request(&request) {
        return Err(map_validation_error(err));
    }

    match state
        .explain_service
        .explain(&request.session_id, request.event_id.as_deref())
    {
        Ok(explanation) => Ok(Json(ExplainResponseV1::from(explanation))),
        Err(err) => Err(map_explain_error(err)),
    }
}

pub async fn get_integrity(
    State(state): State<AppState>,
) -> Result<
    Json<crate::storage::integrity_check::IntegrityReport>,
    (StatusCode, Json<ErrorResponseV1>),
> {
    match state.repo.integrity_check() {
        Ok(report) => Ok(Json(report)),
        Err(err) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(error_response(err.to_string())),
        )),
    }
}

#[derive(serde::Serialize)]
pub struct DebugProjectsResponse {
    pub projects: Vec<String>,
}

pub async fn get_debug_projects(
    State(state): State<AppState>,
) -> Result<Json<DebugProjectsResponse>, (StatusCode, Json<ErrorResponseV1>)> {
    match state.repo.list_projects() {
        Ok(projects) => Ok(Json(DebugProjectsResponse { projects })),
        Err(err) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(error_response(err.to_string())),
        )),
    }
}

fn map_validation_error(err: ValidationError) -> (StatusCode, Json<ErrorResponseV1>) {
    (
        StatusCode::BAD_REQUEST,
        Json(error_response(err.to_string())),
    )
}

fn map_explain_error(err: ExplainError) -> (StatusCode, Json<ErrorResponseV1>) {
    let (status, message) = match &err {
        ExplainError::Storage(StorageError::SessionNotFound(id)) => {
            (StatusCode::NOT_FOUND, format!("session not found: {id}"))
        }
        ExplainError::EventNotFound(id) => {
            (StatusCode::NOT_FOUND, format!("event not found: {id}"))
        }
        ExplainError::Storage(other) => (StatusCode::INTERNAL_SERVER_ERROR, other.to_string()),
    };

    (status, Json(error_response(message)))
}

fn map_error(err: ServiceError) -> (StatusCode, Json<ErrorResponseV1>) {
    let (status, message) = match &err {
        ServiceError::Storage(StorageError::SessionNotFound(id)) => {
            (StatusCode::NOT_FOUND, format!("session not found: {id}"))
        }
        ServiceError::Storage(StorageError::ProjectNotFound(id)) => {
            (StatusCode::NOT_FOUND, format!("project not found: {id}"))
        }
        other => (StatusCode::INTERNAL_SERVER_ERROR, other.to_string()),
    };

    (status, Json(error_response(message)))
}
