use axum::{
    routing::{get, post},
    Router,
};
use tower_http::cors::{Any, CorsLayer};

use super::handlers;
use super::AppState;

pub fn create_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/config", get(handlers::get_config))
        .route("/event", post(handlers::post_event))
        .route("/timeline", get(handlers::get_timeline))
        .route("/explain", post(handlers::post_explain))
        .route("/session/{id}", get(handlers::get_session))
        .route(
            "/project/{id}/sessions",
            get(handlers::get_project_sessions),
        )
        .route("/debug/integrity", get(handlers::get_integrity))
        .route("/debug/projects", get(handlers::get_debug_projects))
        .layer(cors)
        .with_state(state)
}
