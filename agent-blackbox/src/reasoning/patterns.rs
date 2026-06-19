use crate::core::event::Event;
use crate::core::types::EventType;

use super::analyzer::{file_path, test_name};

#[derive(Debug, Clone)]
pub enum PatternKind {
    TestFixCycle {
        failure_idx: usize,
        fix_idxs: Vec<usize>,
        pass_idx: usize,
    },
    ImplementAndVerify {
        change_idxs: Vec<usize>,
        command_idx: Option<usize>,
        verify_idx: usize,
        verify_type: EventType,
    },
    CommitAfterChanges {
        change_idxs: Vec<usize>,
        commit_idx: usize,
    },
    Investigation {
        trigger_idx: usize,
        open_idxs: Vec<usize>,
        modify_idxs: Vec<usize>,
    },
    AgentDirected {
        message_idx: usize,
        action_idxs: Vec<usize>,
    },
}

#[derive(Debug, Clone)]
pub struct DetectedPattern {
    pub kind: PatternKind,
    pub description: String,
}

pub fn detect_patterns(events: &[Event]) -> Vec<DetectedPattern> {
    let mut patterns = Vec::new();

    if let Some(p) = detect_test_fix_cycle(events) {
        patterns.push(p);
    }
    if let Some(p) = detect_investigation(events) {
        patterns.push(p);
    }
    if let Some(p) = detect_implement_and_verify(events) {
        patterns.push(p);
    }
    if let Some(p) = detect_commit_after_changes(events) {
        patterns.push(p);
    }
    if let Some(p) = detect_agent_directed(events) {
        patterns.push(p);
    }

    patterns
}

fn detect_test_fix_cycle(events: &[Event]) -> Option<DetectedPattern> {
    let failure_idx = events
        .iter()
        .position(|e| e.event_type == EventType::TestFailed)?;

    let pass_idx = events
        .iter()
        .enumerate()
        .skip(failure_idx + 1)
        .find(|(_, e)| e.event_type == EventType::TestPassed)?
        .0;

    let fix_idxs: Vec<usize> = events
        .iter()
        .enumerate()
        .filter(|(i, e)| {
            *i > failure_idx
                && *i < pass_idx
                && matches!(
                    e.event_type,
                    EventType::FileModified | EventType::FileCreated | EventType::FileOpened
                )
        })
        .map(|(i, _)| i)
        .collect();

    let test = test_name(&events[failure_idx]).unwrap_or_else(|| "test suite".to_string());
    let mut files: Vec<String> = fix_idxs
        .iter()
        .filter_map(|i| file_path(&events[*i]))
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    files.sort();

    let file_hint = if files.is_empty() {
        "code changes".to_string()
    } else {
        files.join(", ")
    };

    Some(DetectedPattern {
        kind: PatternKind::TestFixCycle {
            failure_idx,
            fix_idxs,
            pass_idx,
        },
        description: format!("test_failed ({test}) → modifications ({file_hint}) → test_passed"),
    })
}

fn detect_investigation(events: &[Event]) -> Option<DetectedPattern> {
    let trigger_idx = events.iter().position(|e| {
        matches!(
            e.event_type,
            EventType::TestFailed | EventType::AgentMessage
        )
    })?;

    let open_idxs: Vec<usize> = events
        .iter()
        .enumerate()
        .skip(trigger_idx + 1)
        .take_while(|(_, e)| e.event_type != EventType::TestPassed)
        .filter(|(_, e)| e.event_type == EventType::FileOpened)
        .map(|(i, _)| i)
        .collect();

    if open_idxs.is_empty() {
        return None;
    }

    let modify_idxs: Vec<usize> = events
        .iter()
        .enumerate()
        .skip(trigger_idx + 1)
        .filter(|(_, e)| {
            matches!(
                e.event_type,
                EventType::FileModified | EventType::FileCreated
            )
        })
        .map(|(i, _)| i)
        .collect();

    let trigger = event_label(&events[trigger_idx]);
    Some(DetectedPattern {
        kind: PatternKind::Investigation {
            trigger_idx,
            open_idxs,
            modify_idxs,
        },
        description: format!("{trigger} → file investigation → code changes"),
    })
}

fn detect_implement_and_verify(events: &[Event]) -> Option<DetectedPattern> {
    let change_idxs: Vec<usize> = events
        .iter()
        .enumerate()
        .filter(|(_, e)| {
            matches!(
                e.event_type,
                EventType::FileModified | EventType::FileCreated
            )
        })
        .map(|(i, _)| i)
        .collect();

    if change_idxs.is_empty() {
        return None;
    }

    let last_change = *change_idxs.last()?;
    let verify = events
        .iter()
        .enumerate()
        .skip(last_change + 1)
        .find(|(_, e)| {
            matches!(
                e.event_type,
                EventType::TestPassed | EventType::TestRun | EventType::CommandExecuted
            )
        })?;

    let command_idx = events
        .iter()
        .enumerate()
        .skip(last_change + 1)
        .take(verify.0.saturating_sub(last_change))
        .find(|(_, e)| e.event_type == EventType::CommandExecuted)
        .map(|(i, _)| i);

    let mut files: Vec<String> = change_idxs
        .iter()
        .filter_map(|i| file_path(&events[*i]))
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    files.sort();

    let file_hint = if files.is_empty() {
        "source files".to_string()
    } else {
        files.join(", ")
    };

    Some(DetectedPattern {
        kind: PatternKind::ImplementAndVerify {
            change_idxs,
            command_idx,
            verify_idx: verify.0,
            verify_type: verify.1.event_type,
        },
        description: format!(
            "file changes ({file_hint}) → {} → {}",
            if command_idx.is_some() {
                "command executed"
            } else {
                "verification"
            },
            verify.1.event_type.as_str()
        ),
    })
}

fn detect_commit_after_changes(events: &[Event]) -> Option<DetectedPattern> {
    let commit_idx = events
        .iter()
        .position(|e| e.event_type == EventType::GitCommit)?;

    let change_idxs: Vec<usize> = events
        .iter()
        .enumerate()
        .take(commit_idx)
        .filter(|(_, e)| {
            matches!(
                e.event_type,
                EventType::FileModified | EventType::FileCreated
            )
        })
        .map(|(i, _)| i)
        .collect();

    if change_idxs.is_empty() {
        return None;
    }

    Some(DetectedPattern {
        kind: PatternKind::CommitAfterChanges {
            change_idxs,
            commit_idx,
        },
        description: "file modifications → git_commit".to_string(),
    })
}

fn detect_agent_directed(events: &[Event]) -> Option<DetectedPattern> {
    let message_idx = events
        .iter()
        .position(|e| e.event_type == EventType::AgentMessage)?;

    let action_idxs: Vec<usize> = events
        .iter()
        .enumerate()
        .skip(message_idx + 1)
        .filter(|(_, e)| {
            !matches!(
                e.event_type,
                EventType::AgentMessage | EventType::FileOpened
            )
        })
        .map(|(i, _)| i)
        .collect();

    if action_idxs.is_empty() {
        return None;
    }

    Some(DetectedPattern {
        kind: PatternKind::AgentDirected {
            message_idx,
            action_idxs,
        },
        description: "agent_message → subsequent actions".to_string(),
    })
}

fn event_label(event: &Event) -> String {
    match event.event_type {
        EventType::TestFailed => {
            let test = test_name(event).unwrap_or_else(|| "unknown test".to_string());
            format!("test_failed ({test})")
        }
        EventType::AgentMessage => {
            let msg = super::analyzer::agent_text(event)
                .map(|t| truncate(&t, 60))
                .unwrap_or_else(|| "agent directive".to_string());
            format!("agent_message ({msg})")
        }
        other => other.as_str().to_string(),
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        format!("{}…", s.chars().take(max).collect::<String>())
    }
}
