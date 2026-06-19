use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpmartoConfig {
    pub api_url: String,
    #[serde(default = "default_project_id")]
    pub project_id: String,
    #[serde(default = "default_true")]
    pub auto_capture: bool,
    #[serde(default = "default_batch_size")]
    pub batch_size: usize,
    #[serde(default = "default_flush_ms")]
    pub flush_interval_ms: u64,
    #[serde(default = "default_retry_max")]
    pub retry_max: u32,
}

fn default_project_id() -> String {
    "auto".into()
}

fn default_true() -> bool {
    true
}

fn default_batch_size() -> usize {
    50
}

fn default_flush_ms() -> u64 {
    2000
}

fn default_retry_max() -> u32 {
    5
}

impl Default for UpmartoConfig {
    fn default() -> Self {
        Self {
            api_url: String::new(),
            project_id: "auto".into(),
            auto_capture: true,
            batch_size: 50,
            flush_interval_ms: 2000,
            retry_max: 5,
        }
    }
}

pub fn global_config_path() -> PathBuf {
    dirs_home().join(".upmarto").join("config.json")
}

pub fn project_config_path(workspace: &Path) -> PathBuf {
    workspace.join(".upmarto").join("config.json")
}

pub fn queue_path(workspace: &Path) -> PathBuf {
    workspace.join(".upmarto").join("queue.jsonl")
}

pub fn runtime_config_path(workspace: &Path) -> PathBuf {
    workspace.join(".upmarto").join("runtime.json")
}

pub fn active_session_path(workspace: &Path) -> PathBuf {
    workspace.join(".upmarto").join("active_session")
}

/// CLI-selected session (workflow isolation). Falls back to daily [`resolve_session`].
pub fn read_active_session(workspace: &Path) -> Option<String> {
    let raw = std::fs::read_to_string(active_session_path(workspace)).ok()?;
    let id = raw.trim().to_string();
    if id.is_empty() { None } else { Some(id) }
}

pub fn write_active_session(workspace: &Path, session_id: &str) -> std::io::Result<()> {
    let dir = workspace.join(".upmarto");
    std::fs::create_dir_all(&dir)?;
    std::fs::write(active_session_path(workspace), format!("{session_id}\n"))
}

/// Unique workflow session id — isolated from the daily workspace session.
pub fn new_workflow_session_id() -> String {
    let ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    format!("wf-{ms}")
}

/// Read `APP_PORT` from workspace `.env` (developer local backend hint).
pub fn read_dotenv_port(workspace: &Path) -> Option<u16> {
    let raw = std::fs::read_to_string(workspace.join(".env")).ok()?;
    for line in raw.lines() {
        let line = line.trim();
        if line.starts_with("APP_PORT=") {
            return line
                .trim_start_matches("APP_PORT=")
                .trim()
                .trim_matches('"')
                .parse()
                .ok();
        }
    }
    None
}

fn dirs_home() -> PathBuf {
    std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."))
}

pub fn load_merged_config(workspace: &Path) -> UpmartoConfig {
    let mut cfg = UpmartoConfig::default();

    if let Some(global) = read_config_file(&global_config_path()) {
        merge_config(&mut cfg, global);
    }
    if let Some(project) = read_config_file(&project_config_path(workspace)) {
        merge_config(&mut cfg, project);
    }

    if let Ok(url) = std::env::var("UPMARTO_URL") {
        if !url.is_empty() {
            cfg.api_url = url.trim_end_matches('/').to_string();
        }
    }

    cfg
}

fn read_config_file(path: &Path) -> Option<UpmartoConfig> {
    let raw = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&raw).ok()
}

fn merge_config(base: &mut UpmartoConfig, overlay: UpmartoConfig) {
    if !overlay.api_url.is_empty() {
        base.api_url = overlay.api_url;
    }
    if overlay.project_id != "auto" {
        base.project_id = overlay.project_id;
    }
    base.auto_capture = overlay.auto_capture;
    base.batch_size = overlay.batch_size;
    base.flush_interval_ms = overlay.flush_interval_ms;
    base.retry_max = overlay.retry_max;
}

pub fn write_project_config(workspace: &Path, cfg: &UpmartoConfig) -> std::io::Result<()> {
    let dir = workspace.join(".upmarto");
    std::fs::create_dir_all(&dir)?;
    let path = dir.join("config.json");
    let json = serde_json::to_string_pretty(cfg)?;
    std::fs::write(path, json)
}

pub fn derive_project_id(workspace: &Path) -> String {
    workspace
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("workspace")
        .replace(|c: char| !c.is_ascii_alphanumeric() && c != '.' && c != '-' && c != '_', "-")
}
