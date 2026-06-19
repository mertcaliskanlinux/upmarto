use std::path::Path;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::config::{project_config_path, read_dotenv_port, runtime_config_path};
use crate::SdkError;

pub const NOT_CONFIGURED_MSG: &str =
    "Upmarto backend not configured. Run: upmarto init";

const PROBE_TIMEOUT: Duration = Duration::from_millis(400);
const SCAN_PORT_START: u16 = 59_000;
const SCAN_PORT_END: u16 = 60_000;
const SCAN_BATCH: usize = 64;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeFile {
    pub api_url: String,
    pub port: u16,
    #[serde(default)]
    pub detected_at: String,
}

/// GET /config — returns true when an Upmarto v1 backend responds.
pub async fn probe_backend(api_url: &str) -> bool {
    let url = format!("{}/config", api_url.trim_end_matches('/'));
    let client = match reqwest::Client::builder().timeout(PROBE_TIMEOUT).build() {
        Ok(c) => c,
        Err(_) => return false,
    };

    let Ok(res) = client.get(&url).send().await else {
        return false;
    };
    if !res.status().is_success() {
        return false;
    }

    let Ok(body) = res.json::<serde_json::Value>().await else {
        return false;
    };

    body.get("api_version")
        .and_then(|v| v.as_str())
        .is_some_and(|v| v == "v1")
}

pub fn read_runtime_file(workspace: &Path) -> Option<RuntimeFile> {
    let raw = std::fs::read_to_string(runtime_config_path(workspace)).ok()?;
    serde_json::from_str(&raw).ok()
}

pub fn write_runtime_file(workspace: &Path, api_url: &str, port: u16) -> std::io::Result<()> {
    let dir = workspace.join(".upmarto");
    std::fs::create_dir_all(&dir)?;
    let runtime = RuntimeFile {
        api_url: api_url.trim_end_matches('/').to_string(),
        port,
        detected_at: chrono::Local::now().to_rfc3339(),
    };
    let json = serde_json::to_string_pretty(&runtime)?;
    std::fs::write(runtime_config_path(workspace), json)
}

fn parse_port_from_url(api_url: &str) -> u16 {
    api_url
        .trim_end_matches('/')
        .rsplit(':')
        .next()
        .and_then(|p| p.parse().ok())
        .unwrap_or(0)
}

/// Discover a live Upmarto backend (runtime.json → .env APP_PORT → port scan).
pub async fn discover_backend(workspace: &Path) -> Result<String, SdkError> {
    if let Some(runtime) = read_runtime_file(workspace) {
        if probe_backend(&runtime.api_url).await {
            return Ok(runtime.api_url);
        }
    }

    if let Some(port) = read_dotenv_port(workspace) {
        let url = format!("http://127.0.0.1:{port}");
        if probe_backend(&url).await {
            return Ok(url);
        }
    }

    if let Some(url) = scan_local_ports().await {
        return Ok(url);
    }

    Err(SdkError::Config(format!(
        "No running Upmarto backend found on 127.0.0.1:{SCAN_PORT_START}-{SCAN_PORT_END}. \
         Start the server (`cargo run`) then run: upmarto init"
    )))
}

async fn scan_local_ports() -> Option<String> {
    let ports: Vec<u16> = (SCAN_PORT_START..=SCAN_PORT_END).collect();

    for chunk in ports.chunks(SCAN_BATCH) {
        let mut tasks = Vec::with_capacity(chunk.len());
        for &port in chunk {
            tasks.push(tokio::spawn(async move {
                let url = format!("http://127.0.0.1:{port}");
                if probe_backend(&url).await {
                    Some(url)
                } else {
                    None
                }
            }));
        }

        for task in tasks {
            if let Ok(Some(url)) = task.await {
                return Some(url);
            }
        }
    }

    None
}

/// Validate workspace config before SDK use. Skips reachability probe when `UPMARTO_URL` is set.
pub async fn validate_workspace_access(workspace: &Path, api_url: &str) -> Result<(), SdkError> {
    if api_url.is_empty() {
        return Err(SdkError::Config(NOT_CONFIGURED_MSG.into()));
    }

    let env_override = std::env::var("UPMARTO_URL")
        .ok()
        .is_some_and(|v| !v.trim().is_empty());

    if !env_override && !project_config_path(workspace).exists() {
        return Err(SdkError::Config(NOT_CONFIGURED_MSG.into()));
    }

    if !env_override && !probe_backend(api_url).await {
        return Err(SdkError::Config(format!(
            "Upmarto backend not reachable at {api_url}. Run: upmarto init"
        )));
    }

    Ok(())
}

/// Bootstrap: discover, probe, write config + runtime.json.
pub async fn bootstrap_workspace(
    workspace: &Path,
    explicit_api_url: Option<String>,
    cfg: &crate::config::UpmartoConfig,
) -> Result<String, SdkError> {
    let api_url = match explicit_api_url {
        Some(url) => url.trim_end_matches('/').to_string(),
        None => std::env::var("UPMARTO_URL")
            .ok()
            .filter(|u| !u.trim().is_empty())
            .map(|u| u.trim_end_matches('/').to_string())
            .unwrap_or(discover_backend(workspace).await?),
    };

    if !probe_backend(&api_url).await {
        return Err(SdkError::Config(format!(
            "Upmarto backend not reachable at {api_url}. \
             Start the server (`cargo run`) then run: upmarto init"
        )));
    }

    let mut final_cfg = cfg.clone();
    final_cfg.api_url = api_url.clone();
    crate::config::write_project_config(workspace, &final_cfg)?;
    let port = parse_port_from_url(&api_url);
    write_runtime_file(workspace, &api_url, port)?;

    std::env::set_var("UPMARTO_URL", &api_url);

    Ok(api_url)
}
