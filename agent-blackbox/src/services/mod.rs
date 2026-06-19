pub mod event_service;
pub mod explain_service;
pub mod session_service;
pub mod timeline_service;

use thiserror::Error;

use crate::storage::StorageError;

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error(transparent)]
    Storage(#[from] StorageError),
}

pub type ServiceResult<T> = Result<T, ServiceError>;
