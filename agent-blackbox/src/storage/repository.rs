use std::path::Path;
use std::sync::Mutex;
use std::time::Duration;

use crate::core::event::Event;
use crate::core::project::Project;
use crate::core::session::Session;

use super::file_store::FileStore;
use super::integrity_check::{repair_missing_indexes, run_integrity_check, IntegrityReport};
use super::sqlite_store::SqliteStore;
use super::StorageResult;

const INDEX_RETRY_ATTEMPTS: u32 = 3;
const INDEX_RETRY_BASE_MS: u64 = 10;

pub struct Repository {
    file_store: Mutex<FileStore>,
    sqlite_store: Mutex<SqliteStore>,
}

impl Repository {
    pub fn new(
        events_log_path: impl AsRef<Path>,
        sqlite_path: impl AsRef<Path>,
    ) -> StorageResult<Self> {
        Ok(Self {
            file_store: Mutex::new(FileStore::new(events_log_path)?),
            sqlite_store: Mutex::new(SqliteStore::new(sqlite_path)?),
        })
    }

    pub fn store_event(&self, event: &Event) -> StorageResult<u64> {
        let sqlite = self.sqlite_store.lock().unwrap();

        sqlite.ensure_project(&event.project_id, event.timestamp)?;
        sqlite.ensure_session(&event.session_id, &event.project_id, event.timestamp)?;

        let offset = {
            let file = self.file_store.lock().unwrap();
            file.append_event(event)?
        };

        match index_event_with_retry(&sqlite, event, offset) {
            Ok(()) => Ok(offset),
            Err(err) => {
                drop(sqlite);
                tracing::warn!(
                    event_id = %event.id,
                    "sqlite index failed after JSONL append; attempting repair"
                );
                let repaired = self.repair_missing_indexes()?;
                if repaired > 0
                    && self
                        .sqlite_store
                        .lock()
                        .unwrap()
                        .event_id_indexed(&event.id.to_string())?
                {
                    Ok(offset)
                } else {
                    Err(err)
                }
            }
        }
    }

    pub fn timeline_for_session(&self, session_id: &str) -> StorageResult<Vec<Event>> {
        let sqlite = self.sqlite_store.lock().unwrap();
        let offsets = sqlite.session_offsets(session_id)?;

        if offsets.is_empty() {
            sqlite.get_session(session_id)?;
            return Ok(Vec::new());
        }

        let file = self.file_store.lock().unwrap();
        file.read_events_at_offsets(&offsets)
    }

    pub fn get_session(&self, session_id: &str) -> StorageResult<Session> {
        self.sqlite_store.lock().unwrap().get_session(session_id)
    }

    pub fn list_project_sessions(&self, project_id: &str) -> StorageResult<Vec<Session>> {
        self.sqlite_store
            .lock()
            .unwrap()
            .list_sessions_for_project(project_id)
    }

    pub fn get_project(&self, project_id: &str) -> StorageResult<Project> {
        self.sqlite_store.lock().unwrap().get_project(project_id)
    }

    pub fn list_projects(&self) -> StorageResult<Vec<String>> {
        self.sqlite_store.lock().unwrap().list_all_project_ids()
    }

    /// Re-indexes JSONL events that are missing from SQLite.
    pub fn repair_missing_indexes(&self) -> StorageResult<u64> {
        let file = self.file_store.lock().unwrap();
        let sqlite = self.sqlite_store.lock().unwrap();
        repair_missing_indexes(&file, &sqlite)
    }

    /// Validates JSONL ↔ SQLite consistency.
    pub fn integrity_check(&self) -> StorageResult<IntegrityReport> {
        let file = self.file_store.lock().unwrap();
        let sqlite = self.sqlite_store.lock().unwrap();
        run_integrity_check(&file, &sqlite)
    }

    /// Runs repair then re-validates; returns the post-repair report.
    pub fn integrity_check_and_repair(&self) -> StorageResult<IntegrityReport> {
        let repaired = self.repair_missing_indexes()?;
        let mut report = self.integrity_check()?;
        report.repaired_count = repaired;
        Ok(report)
    }
}

fn index_event_with_retry(sqlite: &SqliteStore, event: &Event, offset: u64) -> StorageResult<()> {
    let mut last_err = None;
    for attempt in 0..INDEX_RETRY_ATTEMPTS {
        match sqlite.index_event(event, offset) {
            Ok(()) => return Ok(()),
            Err(err) => {
                last_err = Some(err);
                if attempt + 1 < INDEX_RETRY_ATTEMPTS {
                    std::thread::sleep(Duration::from_millis(
                        INDEX_RETRY_BASE_MS * (attempt as u64 + 1),
                    ));
                }
            }
        }
    }
    Err(last_err.expect("retry loop must record at least one error"))
}
