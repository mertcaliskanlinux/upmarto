//! Upmarto Rust SDK — production client for the frozen v1 API.
//!
//! ```no_run
//! use upmarto_sdk::{EventType, TrackEvent, Upmarto};
//! use serde_json::json;
//!
//! #[tokio::main]
//! async fn main() -> upmarto_sdk::Result<()> {
//!     let client = Upmarto::new("http://127.0.0.1:50521", "my-project")?;
//!     client.session("session-abc").await;
//!     client.track(TrackEvent {
//!         event_type: EventType::FileModified,
//!         payload: json!({ "path": "src/main.rs" }),
//!         timestamp: None,
//!     }).await?;
//!     client.flush().await?;
//!     Ok(())
//! }
//! ```

mod bootstrap;
mod client;
mod config;
mod error;
mod session;
mod types;

pub use bootstrap::{
    bootstrap_workspace, discover_backend, probe_backend, write_runtime_file, NOT_CONFIGURED_MSG,
    RuntimeFile,
};
pub use client::Upmarto;
pub use config::{
    active_session_path, derive_project_id, load_merged_config, new_workflow_session_id,
    read_active_session, runtime_config_path, write_active_session, write_project_config,
    UpmartoConfig,
};
pub use error::{Result, SdkError};
pub use session::{derive_session_id, resolve_session};
pub use types::{EventType, ExplainResponse, TimelineResponse, TrackEvent};
