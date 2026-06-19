use std::path::Path;

use chrono::Datelike;
use sha2::{Digest, Sha256};

/// Deterministic session id: hash(workspace_path + YYYY-MM-DD).
pub fn derive_session_id(workspace: &Path, date_key: &str) -> String {
    let normalized = workspace
        .canonicalize()
        .unwrap_or_else(|_| workspace.to_path_buf())
        .to_string_lossy()
        .replace('\\', "/")
        .to_lowercase();

    let hash = Sha256::digest(format!("{normalized}:{date_key}").as_bytes());
    let hex: String = hash.iter().map(|b| format!("{b:02x}")).collect();
    let h = &hex[..32];
    format!(
        "{}-{}-{}-{}-{}",
        &h[0..8],
        &h[8..12],
        &h[12..16],
        &h[16..20],
        &h[20..32]
    )
}

pub fn local_date_key() -> String {
    let now = chrono::Local::now();
    format!("{:04}-{:02}-{:02}", now.year(), now.month(), now.day())
}

/// Resolve session id for a workspace (deterministic per day).
pub fn resolve_session(workspace: &Path) -> String {
    derive_session_id(workspace, &local_date_key())
}
