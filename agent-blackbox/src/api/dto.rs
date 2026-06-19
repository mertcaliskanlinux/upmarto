//! Re-exports frozen product contract types for the HTTP layer.

pub use crate::product::contract::{
    error_response, event_response, session_response, sessions_response, timeline_response,
    validate_create_event, validate_explain_request, validate_path_param, validate_timeline_query,
    ApiErrorBody, CreateEventRequestV1, ErrorResponseV1, EventResponseV1, ExplainRequestV1,
    ExplainResponseV1, SessionResponseV1, SessionsResponseV1, TimelineQueryV1, TimelineResponseV1,
    ValidationError,
};
