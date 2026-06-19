use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use crate::core::event::Event;

use super::{StorageError, StorageResult};

pub struct FileStore {
    path: PathBuf,
}

impl FileStore {
    pub fn new(path: impl AsRef<Path>) -> StorageResult<Self> {
        let path = path.as_ref().to_path_buf();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        if !path.exists() {
            File::create(&path)?;
        }
        Ok(Self { path })
    }

    pub fn append_event(&self, event: &Event) -> StorageResult<u64> {
        let line = serde_json::to_string(event)?;
        let mut file = OpenOptions::new().append(true).open(&self.path)?;

        let offset = file.metadata()?.len();
        writeln!(file, "{line}")?;
        file.sync_data()?;

        Ok(offset)
    }

    pub fn read_event_at(&self, offset: u64) -> StorageResult<Event> {
        let mut file = File::open(&self.path)?;
        file.seek(SeekFrom::Start(offset))?;

        let mut reader = BufReader::new(file);
        let mut line = String::new();
        reader.read_line(&mut line)?;

        if line.is_empty() {
            return Err(StorageError::EventNotFound(offset));
        }

        let event: Event = serde_json::from_str(line.trim_end())?;
        Ok(event)
    }

    pub fn read_events_at_offsets(&self, offsets: &[u64]) -> StorageResult<Vec<Event>> {
        offsets
            .iter()
            .map(|offset| self.read_event_at(*offset))
            .collect()
    }

    pub fn event_count(&self) -> StorageResult<u64> {
        let file = File::open(&self.path)?;
        let reader = BufReader::new(file);
        Ok(reader.lines().count() as u64)
    }

    /// Scans the full JSONL log, returning `(byte_offset, event)` for each non-empty line.
    pub fn scan_all_events(&self) -> StorageResult<Vec<(u64, Event)>> {
        let file = File::open(&self.path)?;
        let reader = BufReader::new(file);
        let mut results = Vec::new();
        let mut offset: u64 = 0;

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                offset += line.len() as u64 + 1;
                continue;
            }
            let event: Event = serde_json::from_str(line.trim_end())?;
            results.push((offset, event));
            offset += line.len() as u64 + 1;
        }

        Ok(results)
    }
}
