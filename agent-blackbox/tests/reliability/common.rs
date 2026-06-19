use std::path::{Path, PathBuf};
use std::process::Output;

use serde_json::json;
use tempfile::TempDir;
use upmarto_sdk::{write_project_config, UpmartoConfig};

pub const DEAD_BACKEND_URL: &str = "http://127.0.0.1:1";

pub fn queue_path(workspace: &Path) -> PathBuf {
    workspace.join(".upmarto").join("queue.jsonl")
}

pub struct Workspace {
    pub _dir: TempDir,
    pub path: PathBuf,
}

impl Workspace {
    pub fn new() -> Self {
        let dir = tempfile::tempdir().expect("temp workspace");
        let path = dir.path().to_path_buf();
        write_workspace_config(&path, DEAD_BACKEND_URL, 1);
        Self { _dir: dir, path }
    }

    pub fn with_api_url(api_url: &str, retry_max: u32) -> Self {
        let dir = tempfile::tempdir().expect("temp workspace");
        let path = dir.path().to_path_buf();
        write_workspace_config(&path, api_url, retry_max);
        Self { _dir: dir, path }
    }
}

pub fn write_workspace_config(workspace: &Path, api_url: &str, retry_max: u32) {
    let cfg = UpmartoConfig {
        api_url: api_url.trim_end_matches('/').to_string(),
        project_id: "auto".into(),
        auto_capture: true,
        batch_size: 50,
        flush_interval_ms: 2000,
        retry_max,
    };
    write_project_config(workspace, &cfg).expect("write config");
    // Explicit env override allows offline/retry tests against unreachable backends.
    std::env::set_var("UPMARTO_URL", api_url.trim_end_matches('/'));
}

pub fn sample_event_line(idx: usize, session_id: &str, project_id: &str) -> String {
    serde_json::to_string(&json!({
        "project_id": project_id,
        "session_id": session_id,
        "event_type": "file_modified",
        "timestamp": 1_700_000_000_000_i64 + idx as i64,
        "payload": { "path": format!("src/file_{idx}.rs") }
    }))
    .expect("serialize event")
}

pub fn write_queue_file(workspace: &Path, lines: &[String]) {
    let path = queue_path(workspace);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("create .upmarto");
    }
    let body = lines.join("\n");
    let content = if body.is_empty() {
        String::new()
    } else {
        format!("{body}\n")
    };
    std::fs::write(&path, content).expect("write queue");
}

pub fn read_queue_file(workspace: &Path) -> Vec<String> {
    let path = queue_path(workspace);
    if !path.exists() {
        return Vec::new();
    }
    std::fs::read_to_string(&path)
        .expect("read queue")
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(str::to_string)
        .collect()
}

pub fn assert_queue_valid_jsonl(lines: &[String]) {
    for (i, line) in lines.iter().enumerate() {
        let value: serde_json::Value = serde_json::from_str(line)
            .unwrap_or_else(|e| panic!("queue line {i} is corrupt JSON: {e}"));
        assert!(
            value.get("project_id").is_some() && value.get("session_id").is_some(),
            "queue line {i} missing required fields"
        );
    }
}

#[allow(dead_code)]
pub fn queue_snapshot(workspace: &Path) -> String {
    read_queue_file(workspace).join("\n")
}

pub fn run_cli(workspace: &Path, args: &[&str]) -> Output {
    let mut cmd = assert_cmd::Command::cargo_bin("upmarto-cli").expect("build upmarto-cli first");
    cmd.arg("--workspace")
        .arg(workspace)
        .args(args)
        .output()
        .expect("spawn upmarto-cli")
}

pub fn stderr_str(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

pub fn stdout_str(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}
