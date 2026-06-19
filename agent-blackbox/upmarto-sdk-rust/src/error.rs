use thiserror::Error;

#[derive(Debug, Error)]
pub enum SdkError {
    #[error("configuration error: {0}")]
    Config(String),
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("api error: {0}")]
    Api(String),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
}

impl SdkError {
    /// Build a user-facing flush failure message; failed events remain queued.
    pub fn flush_failed(
        api_url: &str,
        cause: SdkError,
        flushed: usize,
        remaining: usize,
    ) -> Self {
        let detail = format_flush_cause(&cause);
        SdkError::Api(format!(
            "Upmarto backend is unreachable at {api_url}: {detail} \
             ({flushed} delivered, {remaining} still queued)"
        ))
    }
}

fn format_flush_cause(err: &SdkError) -> String {
    match err {
        SdkError::Http(e) if e.is_connect() => format!("connection failed ({e})"),
        SdkError::Http(e) if e.is_timeout() => format!("request timed out ({e})"),
        SdkError::Http(e) => format!("{e}"),
        other => other.to_string(),
    }
}

pub type Result<T> = std::result::Result<T, SdkError>;
