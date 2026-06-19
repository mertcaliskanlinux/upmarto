use std::path::Path;

use rusqlite::{params, Connection};

use crate::core::event::Event;
use crate::core::session::Session;

use super::{StorageError, StorageResult};

pub struct SqliteStore {
    conn: Connection,
}

impl SqliteStore {
    pub fn new(path: impl AsRef<Path>) -> StorageResult<Self> {
        if let Some(parent) = path.as_ref().parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(path)?;
        let store = Self { conn };
        store.init_schema()?;
        Ok(store)
    }

    fn init_schema(&self) -> StorageResult<()> {
        self.conn.execute_batch(
            "
            PRAGMA journal_mode = WAL;
            PRAGMA synchronous = NORMAL;

            CREATE TABLE IF NOT EXISTS projects (
                id TEXT PRIMARY KEY,
                created_at INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                project_id TEXT NOT NULL,
                started_at INTEGER NOT NULL,
                ended_at INTEGER,
                event_count INTEGER NOT NULL DEFAULT 0,
                FOREIGN KEY (project_id) REFERENCES projects(id)
            );

            CREATE INDEX IF NOT EXISTS idx_sessions_project_id ON sessions(project_id);

            CREATE TABLE IF NOT EXISTS event_offsets (
                event_id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL,
                file_offset INTEGER NOT NULL,
                timestamp INTEGER NOT NULL,
                FOREIGN KEY (session_id) REFERENCES sessions(id)
            );

            CREATE INDEX IF NOT EXISTS idx_event_offsets_session_ts
                ON event_offsets(session_id, timestamp);
            ",
        )?;
        Ok(())
    }

    pub fn ensure_project(&self, project_id: &str, created_at: i64) -> StorageResult<()> {
        self.conn.execute(
            "INSERT OR IGNORE INTO projects (id, created_at) VALUES (?1, ?2)",
            params![project_id, created_at],
        )?;
        Ok(())
    }

    pub fn ensure_session(
        &self,
        session_id: &str,
        project_id: &str,
        timestamp: i64,
    ) -> StorageResult<()> {
        self.conn.execute(
            "INSERT OR IGNORE INTO sessions (id, project_id, started_at, event_count)
             VALUES (?1, ?2, ?3, 0)",
            params![session_id, project_id, timestamp],
        )?;
        Ok(())
    }

    pub fn index_event(&self, event: &Event, file_offset: u64) -> StorageResult<()> {
        let tx = self.conn.unchecked_transaction()?;

        tx.execute(
            "INSERT INTO event_offsets (event_id, session_id, file_offset, timestamp)
             VALUES (?1, ?2, ?3, ?4)",
            params![
                event.id.to_string(),
                event.session_id,
                file_offset as i64,
                event.timestamp
            ],
        )?;

        tx.execute(
            "UPDATE sessions
             SET event_count = event_count + 1,
                 ended_at = MAX(COALESCE(ended_at, 0), ?1)
             WHERE id = ?2",
            params![event.timestamp, event.session_id],
        )?;

        tx.commit()?;
        Ok(())
    }

    pub fn event_id_indexed(&self, event_id: &str) -> StorageResult<bool> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM event_offsets WHERE event_id = ?1",
            params![event_id],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    pub fn indexed_event_count(&self) -> StorageResult<u64> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM event_offsets", [], |row| row.get(0))?;
        Ok(count as u64)
    }

    pub fn all_indexed_offsets(&self) -> StorageResult<Vec<(String, u64)>> {
        let mut stmt = self
            .conn
            .prepare("SELECT event_id, file_offset FROM event_offsets ORDER BY rowid ASC")?;
        let rows = stmt
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)? as u64))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn session_event_counts(&self) -> StorageResult<Vec<(String, i64)>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, event_count FROM sessions ORDER BY id ASC")?;
        let rows = stmt
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn count_offsets_for_session(&self, session_id: &str) -> StorageResult<i64> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM event_offsets WHERE session_id = ?1",
            params![session_id],
            |row| row.get(0),
        )?;
        Ok(count)
    }

    pub fn session_offsets(&self, session_id: &str) -> StorageResult<Vec<u64>> {
        let mut stmt = self.conn.prepare(
            "SELECT file_offset FROM event_offsets
             WHERE session_id = ?1
             ORDER BY timestamp ASC, rowid ASC",
        )?;

        let offsets = stmt
            .query_map(params![session_id], |row| row.get::<_, i64>(0))?
            .map(|r| r.map(|o| o as u64))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(offsets)
    }

    pub fn get_session(&self, session_id: &str) -> StorageResult<Session> {
        let mut stmt = self.conn.prepare(
            "SELECT id, project_id, started_at, ended_at, event_count
             FROM sessions WHERE id = ?1",
        )?;

        let session = stmt
            .query_row(params![session_id], |row| {
                Ok(Session {
                    id: row.get(0)?,
                    project_id: row.get(1)?,
                    started_at: row.get(2)?,
                    ended_at: row.get(3)?,
                    event_count: row.get(4)?,
                })
            })
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => {
                    StorageError::SessionNotFound(session_id.to_string())
                }
                other => StorageError::Sqlite(other),
            })?;

        Ok(session)
    }

    pub fn list_sessions_for_project(&self, project_id: &str) -> StorageResult<Vec<Session>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, project_id, started_at, ended_at, event_count
             FROM sessions
             WHERE project_id = ?1
             ORDER BY started_at DESC",
        )?;

        let sessions = stmt
            .query_map(params![project_id], |row| {
                Ok(Session {
                    id: row.get(0)?,
                    project_id: row.get(1)?,
                    started_at: row.get(2)?,
                    ended_at: row.get(3)?,
                    event_count: row.get(4)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        if sessions.is_empty() && !self.project_exists(project_id)? {
            return Err(StorageError::ProjectNotFound(project_id.to_string()));
        }

        Ok(sessions)
    }

    fn project_exists(&self, project_id: &str) -> StorageResult<bool> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM projects WHERE id = ?1",
            params![project_id],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    pub fn list_all_project_ids(&self) -> StorageResult<Vec<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id FROM projects ORDER BY created_at DESC")?;
        let ids = stmt
            .query_map([], |row| row.get::<_, String>(0))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(ids)
    }

    pub fn get_project(&self, project_id: &str) -> StorageResult<crate::core::project::Project> {
        let mut stmt = self.conn.prepare(
            "SELECT p.id, p.created_at,
                    (SELECT COUNT(*) FROM sessions s WHERE s.project_id = p.id) as session_count
             FROM projects p WHERE p.id = ?1",
        )?;

        stmt.query_row(params![project_id], |row| {
            Ok(crate::core::project::Project {
                id: row.get(0)?,
                created_at: row.get(1)?,
                session_count: row.get(2)?,
            })
        })
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => {
                StorageError::ProjectNotFound(project_id.to_string())
            }
            other => StorageError::Sqlite(other),
        })
    }
}
