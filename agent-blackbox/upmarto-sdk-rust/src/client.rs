use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::Mutex;
use tokio::time::{interval, sleep};

use crate::config::{load_merged_config, queue_path, UpmartoConfig};
use crate::types::{
    CreateEventRequest, ExplainRequest, ExplainResponse, TimelineResponse, TrackEvent,
};
use crate::SdkError;

const DEFAULT_TIMEOUT_SECS: u64 = 10;

struct QueuedEvent {
    request: CreateEventRequest,
    attempts: u32,
}

/// Production-safe async client for the Upmarto v1 API.
pub struct Upmarto {
    api_url: String,
    project_id: String,
    session_id: Mutex<String>,
    client: reqwest::Client,
    queue: Arc<Mutex<Vec<QueuedEvent>>>,
    config: UpmartoConfig,
    queue_file: Option<PathBuf>,
}

impl Upmarto {
    /// Create a new client. `project_id` can be `"auto"` to derive from workspace later.
    pub fn new(api_url: impl Into<String>, project_id: impl Into<String>) -> Result<Self, SdkError> {
        let api_url = api_url.into().trim_end_matches('/').to_string();
        if api_url.is_empty() {
            return Err(SdkError::Config("api_url must not be empty".into()));
        }

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS))
            .build()?;

        Ok(Self {
            api_url,
            project_id: project_id.into(),
            session_id: Mutex::new(String::new()),
            client,
            queue: Arc::new(Mutex::new(Vec::new())),
            config: UpmartoConfig::default(),
            queue_file: None,
        })
    }

    /// Load config from `.upmarto/config.json` + `~/.upmarto/config.json`.
    pub async fn from_workspace(workspace: impl AsRef<Path>) -> Result<Self, SdkError> {
        let workspace = workspace.as_ref();
        let cfg = load_merged_config(workspace);
        if cfg.api_url.is_empty() {
            return Err(SdkError::Config(crate::bootstrap::NOT_CONFIGURED_MSG.into()));
        }

        crate::bootstrap::validate_workspace_access(workspace, &cfg.api_url).await?;

        let project_id = if cfg.project_id == "auto" {
            crate::config::derive_project_id(workspace)
        } else {
            cfg.project_id.clone()
        };

        let mut client = Self::new(cfg.api_url.clone(), project_id)?;
        client.config = cfg;
        client.queue_file = Some(queue_path(workspace));
        Ok(client)
    }

    /// Override the active session id.
    pub async fn session(&self, session_id: impl Into<String>) {
        *self.session_id.lock().await = session_id.into();
    }

    pub fn api_url(&self) -> &str {
        &self.api_url
    }

    pub fn project_id(&self) -> &str {
        &self.project_id
    }

    pub async fn current_session_id(&self) -> String {
        self.session_id.lock().await.clone()
    }

    /// Enqueue an event for batched delivery (non-blocking).
    pub async fn track(&self, event: TrackEvent) -> Result<(), SdkError> {
        let session_id = self.session_id.lock().await.clone();
        if session_id.is_empty() {
            return Err(SdkError::Config(
                "session_id not set — call session() first".into(),
            ));
        }

        let request = CreateEventRequest {
            project_id: self.project_id.clone(),
            session_id,
            event_type: event.event_type,
            timestamp: event.timestamp,
            payload: event.payload,
        };

        self.enqueue(request).await
    }

    async fn enqueue(&self, request: CreateEventRequest) -> Result<(), SdkError> {
        if let Some(path) = &self.queue_file {
            persist_event(path, &request)?;
        }

        let mut queue = self.queue.lock().await;
        queue.push(QueuedEvent {
            request,
            attempts: 0,
        });

        if queue.len() >= self.config.batch_size {
            drop(queue);
            self.flush().await?;
        }

        Ok(())
    }

    /// Flush all queued events to `POST /event`.
    /// Returns the number of events successfully delivered.
    /// On failure, undelivered events are kept in the queue (and on disk when persisted).
    pub async fn flush(&self) -> Result<usize, SdkError> {
        let mut total_flushed = 0usize;

        loop {
            let batch: Vec<QueuedEvent> = {
                let mut queue = self.queue.lock().await;
                if queue.is_empty() {
                    return Ok(total_flushed);
                }
                let take = queue.len().min(self.config.batch_size);
                queue.drain(..take).collect()
            };

            total_flushed += self.flush_batch(batch).await?;
        }
    }

    async fn flush_batch(&self, batch: Vec<QueuedEvent>) -> Result<usize, SdkError> {
        let max_attempts = self.config.retry_max.max(1);
        let mut pending = batch;

        for attempt in 0..max_attempts {
            let mut failed = Vec::new();
            let mut flushed = 0usize;
            let mut last_err: Option<SdkError> = None;

            for item in pending {
                match self.post_event(&item.request).await {
                    Ok(()) => {
                        flushed += 1;
                        if let Some(path) = &self.queue_file {
                            let _ = remove_persisted(path, &item.request);
                        }
                    }
                    Err(err) => {
                        last_err = Some(err);
                        failed.push(QueuedEvent {
                            attempts: item.attempts + 1,
                            request: item.request,
                        });
                    }
                }
            }

            if failed.is_empty() {
                return Ok(flushed);
            }

            if attempt + 1 < max_attempts {
                let delay = 250 * 2u64.pow(attempt);
                tracing::warn!(
                    "flush retry {}/{} ({} events pending): {}",
                    attempt + 1,
                    max_attempts,
                    failed.len(),
                    last_err.as_ref().map(ToString::to_string).unwrap_or_default()
                );
                sleep(Duration::from_millis(delay)).await;
                pending = failed;
                continue;
            }

            let remaining = {
                let mut queue = self.queue.lock().await;
                for item in failed.into_iter().rev() {
                    queue.insert(0, item);
                }
                queue.len()
            };

            return Err(SdkError::flush_failed(
                &self.api_url,
                last_err.unwrap_or_else(|| SdkError::Api("unknown error".into())),
                flushed,
                remaining,
            ));
        }

        Ok(0)
    }

    /// Background flusher — call once at startup.
    pub fn spawn_auto_flush(self: &Arc<Self>) {
        let client = Arc::clone(self);
        let period = self.config.flush_interval_ms;
        tokio::spawn(async move {
            let mut tick = interval(Duration::from_millis(period));
            loop {
                tick.tick().await;
                if let Err(err) = client.flush().await {
                    tracing::error!("background flush failed: {err}");
                }
            }
        });
    }

    /// Reload persisted offline queue from disk.
    pub async fn restore_persisted_queue(&self) -> Result<usize, SdkError> {
        let Some(path) = self.queue_file.as_ref() else {
            return Ok(0);
        };
        if !path.exists() {
            return Ok(0);
        }

        let raw = std::fs::read_to_string(path)?;
        let mut restored = 0usize;
        let mut queue = self.queue.lock().await;

        for line in raw.lines() {
            if line.trim().is_empty() {
                continue;
            }
            if let Ok(req) = serde_json::from_str::<CreateEventRequest>(line) {
                queue.push(QueuedEvent {
                    request: req,
                    attempts: 0,
                });
                restored += 1;
            }
        }

        Ok(restored)
    }

    async fn post_event(&self, request: &CreateEventRequest) -> Result<(), SdkError> {
        let res = self
            .client
            .post(format!("{}/event", self.api_url))
            .json(request)
            .send()
            .await?;

        if !res.status().is_success() {
            let status = res.status();
            let body = res.text().await.unwrap_or_default();
            return Err(SdkError::Api(format!(
                "POST /event {status}: {}",
                body.chars().take(200).collect::<String>()
            )));
        }
        Ok(())
    }

    /// `POST /explain` — direct API call (not queued).
    pub async fn explain(&self, session_id: &str) -> Result<ExplainResponse, SdkError> {
        let res = self
            .client
            .post(format!("{}/explain", self.api_url))
            .json(&ExplainRequest {
                session_id: session_id.to_string(),
                event_id: None,
            })
            .send()
            .await?;

        if !res.status().is_success() {
            let status = res.status();
            let body = res.text().await.unwrap_or_default();
            return Err(SdkError::Api(format!(
                "POST /explain {status}: {}",
                body.chars().take(200).collect::<String>()
            )));
        }

        Ok(res.json().await?)
    }

    /// `GET /timeline` — direct API call.
    pub async fn timeline(&self, session_id: &str) -> Result<TimelineResponse, SdkError> {
        let res = self
            .client
            .get(format!("{}/timeline", self.api_url))
            .query(&[("session_id", session_id)])
            .send()
            .await?;

        if !res.status().is_success() {
            let status = res.status();
            let body = res.text().await.unwrap_or_default();
            return Err(SdkError::Api(format!(
                "GET /timeline {status}: {}",
                body.chars().take(200).collect::<String>()
            )));
        }

        Ok(res.json().await?)
    }

    /// `GET /config` — discover API base URL.
    pub async fn discover_config(&self) -> Result<serde_json::Value, SdkError> {
        let res = self
            .client
            .get(format!("{}/config", self.api_url))
            .send()
            .await?;
        Ok(res.json().await?)
    }
}

fn persist_event(path: &Path, request: &CreateEventRequest) -> Result<(), SdkError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let line = serde_json::to_string(request)?;
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    writeln!(file, "{line}")?;
    Ok(())
}

fn remove_persisted(path: &Path, sent: &CreateEventRequest) -> Result<(), SdkError> {
    if !path.exists() {
        return Ok(());
    }
    let target = serde_json::to_string(sent)?;
    let remaining: Vec<String> = std::fs::read_to_string(path)?
        .lines()
        .filter(|l| !l.trim().is_empty() && *l != target)
        .map(str::to_string)
        .collect();
    if remaining.is_empty() {
        let _ = std::fs::remove_file(path);
    } else {
        std::fs::write(path, format!("{}\n", remaining.join("\n")))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::EventType;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn flush_returns_count_on_success() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/event"))
            .respond_with(ResponseTemplate::new(201))
            .expect(2)
            .mount(&server)
            .await;

        let client = Upmarto::new(server.uri(), "proj").unwrap();
        client.session("sess").await;

        for _ in 0..2 {
            client
                .track(TrackEvent {
                    event_type: EventType::FileModified,
                    payload: serde_json::json!({ "path": "a.rs" }),
                    timestamp: None,
                })
                .await
                .unwrap();
        }

        let flushed = client.flush().await.unwrap();
        assert_eq!(flushed, 2);
    }

    #[tokio::test]
    async fn flush_returns_err_when_backend_unreachable() {
        let client = Upmarto::new("http://127.0.0.1:1", "proj").unwrap();
        client.session("sess").await;
        client
            .track(TrackEvent {
                event_type: EventType::FileModified,
                payload: serde_json::json!({ "path": "a.rs" }),
                timestamp: None,
            })
            .await
            .unwrap();

        let err = client.flush().await.unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("127.0.0.1:1"), "unexpected message: {msg}");
        assert!(msg.contains("still queued"), "unexpected message: {msg}");

        assert_eq!(client.queue.lock().await.len(), 1);
    }

    #[tokio::test]
    async fn flush_returns_err_on_http_error_and_keeps_queue() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/event"))
            .respond_with(ResponseTemplate::new(503).set_body_string("service unavailable"))
            .mount(&server)
            .await;

        let mut client = Upmarto::new(server.uri(), "proj").unwrap();
        client.config.retry_max = 1;
        client.session("sess").await;
        client
            .track(TrackEvent {
                event_type: EventType::FileModified,
                payload: serde_json::json!({ "path": "a.rs" }),
                timestamp: None,
            })
            .await
            .unwrap();

        let err = client.flush().await.unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("503"), "unexpected message: {msg}");
        assert!(msg.contains("still queued"), "unexpected message: {msg}");
        assert_eq!(client.queue.lock().await.len(), 1);
    }
}
