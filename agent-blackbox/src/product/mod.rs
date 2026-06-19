//! # Upmarto — Product Layer (FROZEN)
//!
//! This module defines the public product contract. The reasoning engine,
//! storage layer, and test infrastructure are frozen. Changes here must be
//! additive or versioned (v2+).
//!
//! ## Product definition
//!
//! Upmarto is a local-first AI event intelligence system — **Memory and
//! Reasoning for AI Agents** — that records coding agent behavior and explains
//! **why** actions happened.
//!
//! ## What this system IS
//!
//! - AI agent behavior debugger
//! - Causal reasoning engine for developer actions
//! - Session replay + explanation system
//!
//! ## What this system IS NOT
//!
//! - A logging system
//! - A telemetry pipeline
//! - A generic event store
//!
//! ## Hard rules
//!
//! - No new event types without `EVENT_SCHEMA_VERSION` bump
//! - No `/explain` schema changes without `EXPLAIN_SCHEMA_VERSION` bump
//! - No storage redesign (JSONL + SQLite)
//! - No LLM dependency
//! - No external services — local-first only

pub mod contract;
pub mod views;

pub use contract::{
    validate_create_event, validate_explain_request, validate_path_param, validate_timeline_query,
    ApiErrorBody, CreateEventRequestV1, ErrorResponseV1, EventResponseV1, ExplainRequestV1,
    ExplainResponseV1, SessionResponseV1, SessionsResponseV1, TimelineQueryV1, TimelineResponseV1,
    ValidationError, API_VERSION, EVENT_SCHEMA_VERSION, EVENT_TYPES_V1, EXPLAIN_SCHEMA_VERSION,
    PRODUCT_NAME, PRODUCT_TAGLINE,
};
pub use views::{AnalysisUnit, ExplainView, ProductSurface, SessionView, TimelineView};
