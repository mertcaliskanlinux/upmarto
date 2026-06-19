pub mod file_store;
pub mod integrity_check;
pub mod repository;
pub mod sqlite_store;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("session not found: {0}")]
    SessionNotFound(String),
    #[error("project not found: {0}")]
    ProjectNotFound(String),
    #[error("event not found at offset {0}")]
    EventNotFound(u64),
}

pub type StorageResult<T> = Result<T, StorageError>;
