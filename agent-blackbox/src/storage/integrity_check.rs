use serde::Serialize;

use super::file_store::FileStore;
use super::sqlite_store::SqliteStore;
use super::StorageResult;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum IntegrityStatus {
    Ok,
    Fail,
}

#[derive(Debug, Clone, Serialize)]
pub struct IntegrityReport {
    pub status: IntegrityStatus,
    pub jsonl_line_count: u64,
    pub sqlite_indexed_count: u64,
    pub orphan_jsonl_count: u64,
    pub missing_offset_count: u64,
    pub broken_session_ids: Vec<String>,
    pub ordering_issue_count: u64,
    pub repaired_count: u64,
}

pub fn run_integrity_check(
    file: &FileStore,
    sqlite: &SqliteStore,
) -> StorageResult<IntegrityReport> {
    let jsonl_events = file.scan_all_events()?;
    let jsonl_line_count = jsonl_events.len() as u64;
    let sqlite_indexed_count = sqlite.indexed_event_count()?;

    let mut orphan_jsonl_count = 0u64;
    for (_, event) in &jsonl_events {
        if !sqlite.event_id_indexed(&event.id.to_string())? {
            orphan_jsonl_count += 1;
        }
    }

    let mut missing_offset_count = 0u64;
    for (_, offset) in sqlite.all_indexed_offsets()? {
        if file.read_event_at(offset).is_err() {
            missing_offset_count += 1;
        }
    }

    let mut broken_session_ids = Vec::new();
    for (session_id, declared_count) in sqlite.session_event_counts()? {
        let actual = sqlite.count_offsets_for_session(&session_id)?;
        if actual != declared_count {
            broken_session_ids.push(session_id);
        }
    }

    let mut ordering_issue_count = 0u64;
    let mut sessions_checked = std::collections::HashSet::new();
    for (_, event) in &jsonl_events {
        if !sessions_checked.insert(event.session_id.clone()) {
            continue;
        }
        let offsets = sqlite.session_offsets(&event.session_id)?;
        if offsets.len() < 2 {
            continue;
        }
        let events = file.read_events_at_offsets(&offsets)?;
        for window in events.windows(2) {
            if window[0].timestamp > window[1].timestamp {
                ordering_issue_count += 1;
            }
        }
    }

    let status = if orphan_jsonl_count == 0
        && missing_offset_count == 0
        && broken_session_ids.is_empty()
        && ordering_issue_count == 0
        && jsonl_line_count == sqlite_indexed_count
    {
        IntegrityStatus::Ok
    } else {
        IntegrityStatus::Fail
    };

    Ok(IntegrityReport {
        status,
        jsonl_line_count,
        sqlite_indexed_count,
        orphan_jsonl_count,
        missing_offset_count,
        broken_session_ids,
        ordering_issue_count,
        repaired_count: 0,
    })
}

pub fn repair_missing_indexes(file: &FileStore, sqlite: &SqliteStore) -> StorageResult<u64> {
    let jsonl_events = file.scan_all_events()?;
    let mut repaired = 0u64;

    for (offset, event) in jsonl_events {
        if sqlite.event_id_indexed(&event.id.to_string())? {
            continue;
        }
        sqlite.ensure_project(&event.project_id, event.timestamp)?;
        sqlite.ensure_session(&event.session_id, &event.project_id, event.timestamp)?;
        sqlite.index_event(&event, offset)?;
        repaired += 1;
    }

    Ok(repaired)
}
