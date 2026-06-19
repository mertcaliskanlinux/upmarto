use std::env;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};

/// Type-safe application settings loaded from environment variables.
#[derive(Debug, Clone)]
pub struct Settings {
    pub host: String,
    /// Bind port. `0` selects a free port at runtime (OS-assigned).
    pub port: u16,
    pub events_log_path: PathBuf,
    pub sqlite_path: PathBuf,
    pub data_dir: PathBuf,
    pub test_mode: bool,
    /// Optional public URL override (reverse proxy / container ingress).
    pub public_base_url: Option<String>,
}

impl Settings {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();

        let test_mode = env_flag("TEST_MODE");
        let data_dir = resolve_data_dir(test_mode);

        let events_log_path = env_path("DATABASE_PATH")
            .or_else(|| env_path("EVENTS_LOG_PATH"))
            .unwrap_or_else(|| data_dir.join("events.log"));

        let sqlite_path = env_path("SQLITE_PATH").unwrap_or_else(|| data_dir.join("metadata.db"));

        let data_dir = events_log_path
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or(data_dir);

        Self {
            host: env::var("APP_HOST")
                .or_else(|_| env::var("HOST"))
                .unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: env::var("APP_PORT")
                .or_else(|_| env::var("PORT"))
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(0),
            events_log_path,
            sqlite_path,
            data_dir,
            test_mode,
            public_base_url: env::var("PUBLIC_BASE_URL")
                .ok()
                .filter(|s| !s.trim().is_empty()),
        }
    }

    /// Isolated storage for a single test instance (temp dir + ephemeral port).
    pub fn for_isolated_test(data_dir: impl AsRef<Path>, port: u16) -> Self {
        let data_dir = data_dir.as_ref().to_path_buf();
        Self {
            host: "127.0.0.1".to_string(),
            port,
            events_log_path: data_dir.join("events.log"),
            sqlite_path: data_dir.join("metadata.db"),
            data_dir: data_dir.clone(),
            test_mode: true,
            public_base_url: None,
        }
    }

    pub fn events_log_path(&self) -> &Path {
        &self.events_log_path
    }

    pub fn sqlite_path(&self) -> &Path {
        &self.sqlite_path
    }

    /// Create parent directories for storage files (safe for custom DATABASE_PATH / SQLITE_PATH).
    pub fn ensure_storage_dirs(&self) -> std::io::Result<()> {
        if let Some(parent) = self.events_log_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        if let Some(parent) = self.sqlite_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        Ok(())
    }

    /// Build the client-facing API base URL after the listener is bound.
    pub fn resolve_api_base_url(bound: SocketAddr, public_override: &Option<String>) -> String {
        if let Some(url) = public_override {
            return url.trim_end_matches('/').to_string();
        }

        let host = if bound.ip().is_unspecified() {
            "127.0.0.1".to_string()
        } else {
            bound.ip().to_string()
        };

        format!("http://{host}:{}", bound.port())
    }
}

fn env_flag(name: &str) -> bool {
    env::var(name)
        .map(|v| v == "true" || v == "1")
        .unwrap_or(false)
}

fn env_path(name: &str) -> Option<PathBuf> {
    env::var(name)
        .ok()
        .map(PathBuf::from)
        .filter(|p| !p.as_os_str().is_empty())
}

fn resolve_data_dir(test_mode: bool) -> PathBuf {
    if let Some(dir) = env_path("DATA_DIR") {
        return dir;
    }

    if test_mode {
        PathBuf::from("./data/test")
    } else {
        PathBuf::from("./data")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_api_base_url_unspecified_host_uses_loopback() {
        let bound: SocketAddr = "0.0.0.0:54321".parse().unwrap();
        assert_eq!(
            Settings::resolve_api_base_url(bound, &None),
            "http://127.0.0.1:54321"
        );
    }

    #[test]
    fn public_base_url_override() {
        let bound: SocketAddr = "0.0.0.0:8080".parse().unwrap();
        let url = Some("https://api.example.com".to_string());
        assert_eq!(
            Settings::resolve_api_base_url(bound, &url),
            "https://api.example.com"
        );
    }
}
