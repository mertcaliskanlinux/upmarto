use std::collections::HashSet;

use serde_json::Value;

use crate::core::event::Event;
use crate::core::types::EventType;

use super::patterns::{detect_patterns, DetectedPattern};

#[derive(Debug, Clone)]
pub struct SessionAnalysis {
    pub event_count: usize,
    pub patterns: Vec<DetectedPattern>,
    pub affected_files: Vec<String>,
    pub commands: Vec<String>,
    pub failed_tests: Vec<String>,
    pub passed_tests: Vec<String>,
    pub agent_messages: Vec<String>,
    pub commit_messages: Vec<String>,
    pub primary_trigger: Option<TriggerKind>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum TriggerPriority {
    FileChange = 1,
    Command = 2,
    GitCommit = 3,
    TestPassed = 4,
    AgentDirective = 5,
    TestFailure = 6,
}

#[derive(Debug, Clone)]
pub enum TriggerKind {
    TestFailure(String),
    TestPassed(String),
    GitCommit(String),
    Command(String),
    AgentDirective(String),
    FileChange(String),
}

pub fn analyze(events: &[Event]) -> SessionAnalysis {
    let patterns = detect_patterns(events);

    let mut affected_files: Vec<String> = events
        .iter()
        .filter_map(file_path)
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    affected_files.sort();

    let commands: Vec<String> = events
        .iter()
        .filter(|e| e.event_type == EventType::CommandExecuted)
        .filter_map(command_text)
        .collect();

    let failed_tests: Vec<String> = events
        .iter()
        .filter(|e| e.event_type == EventType::TestFailed)
        .filter_map(test_name)
        .collect();

    let passed_tests: Vec<String> = events
        .iter()
        .filter(|e| e.event_type == EventType::TestPassed)
        .filter_map(test_name)
        .collect();

    let agent_messages: Vec<String> = events
        .iter()
        .filter(|e| e.event_type == EventType::AgentMessage)
        .filter_map(agent_text)
        .collect();

    let commit_messages: Vec<String> = events
        .iter()
        .filter(|e| e.event_type == EventType::GitCommit)
        .filter_map(commit_message)
        .collect();

    let primary_trigger = infer_primary_trigger(events);

    SessionAnalysis {
        event_count: events.len(),
        patterns,
        affected_files,
        commands,
        failed_tests,
        passed_tests,
        agent_messages,
        commit_messages,
        primary_trigger,
    }
}

/// Selects the highest-priority analytical anchor across the full session.
/// `test_failed` always wins when present — never overridden by earlier file edits.
fn infer_primary_trigger(events: &[Event]) -> Option<TriggerKind> {
    let mut best: Option<(TriggerPriority, TriggerKind)> = None;

    for event in events {
        if let Some((priority, kind)) = trigger_from_event(event) {
            if best.as_ref().is_none_or(|(p, _)| priority > *p) {
                best = Some((priority, kind));
            }
        }
    }

    best.map(|(_, kind)| kind)
}

fn trigger_from_event(event: &Event) -> Option<(TriggerPriority, TriggerKind)> {
    match event.event_type {
        EventType::TestFailed => Some((
            TriggerPriority::TestFailure,
            TriggerKind::TestFailure(
                test_name(event).unwrap_or_else(|| "unknown test".to_string()),
            ),
        )),
        EventType::TestPassed => Some((
            TriggerPriority::TestPassed,
            TriggerKind::TestPassed(test_name(event).unwrap_or_else(|| "unknown test".to_string())),
        )),
        EventType::GitCommit => commit_message(event)
            .map(|msg| (TriggerPriority::GitCommit, TriggerKind::GitCommit(msg))),
        EventType::CommandExecuted => {
            command_text(event).map(|cmd| (TriggerPriority::Command, TriggerKind::Command(cmd)))
        }
        EventType::FileModified | EventType::FileCreated => file_path(event)
            .map(|path| (TriggerPriority::FileChange, TriggerKind::FileChange(path))),
        EventType::AgentMessage => agent_text(event).map(|text| {
            (
                TriggerPriority::AgentDirective,
                TriggerKind::AgentDirective(text),
            )
        }),
        _ => None,
    }
}

pub fn file_path(event: &Event) -> Option<String> {
    payload_str(&event.payload, &["path", "file", "filepath", "filename"])
}

pub fn command_text(event: &Event) -> Option<String> {
    payload_str(&event.payload, &["command", "cmd", "shell"])
}

pub fn test_name(event: &Event) -> Option<String> {
    payload_str(
        &event.payload,
        &["test", "name", "suite", "test_name", "spec"],
    )
}

pub fn agent_text(event: &Event) -> Option<String> {
    payload_str(
        &event.payload,
        &["message", "content", "text", "reasoning", "intent"],
    )
}

pub fn commit_message(event: &Event) -> Option<String> {
    payload_str(
        &event.payload,
        &["message", "commit_message", "summary", "description"],
    )
}

pub fn error_detail(event: &Event) -> Option<String> {
    payload_str(
        &event.payload,
        &["error", "failure", "reason", "output", "stderr"],
    )
}

fn payload_str(payload: &Value, keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|key| {
        payload.get(*key).and_then(|v| match v {
            Value::String(s) if !s.is_empty() => Some(s.clone()),
            _ => None,
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::event::Event;
    use serde_json::json;
    use uuid::Uuid;

    fn make_event(event_type: EventType, payload: serde_json::Value) -> Event {
        Event {
            id: Uuid::new_v4(),
            timestamp: 0,
            project_id: "p".into(),
            session_id: "s".into(),
            event_type,
            payload,
        }
    }

    #[test]
    fn primary_trigger_prefers_test_failed_over_earlier_file_change() {
        let events = vec![
            make_event(EventType::FileModified, json!({ "path": "auth.go" })),
            make_event(
                EventType::TestFailed,
                json!({ "test": "auth_session_expiry", "error": "token expired" }),
            ),
        ];

        let analysis = analyze(&events);
        match analysis.primary_trigger {
            Some(TriggerKind::TestFailure(name)) => {
                assert_eq!(name, "auth_session_expiry");
            }
            other => panic!("expected TestFailure, got {other:?}"),
        }
    }
}
