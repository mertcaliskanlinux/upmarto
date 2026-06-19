use serde::Serialize;

use crate::core::event::Event;
use crate::core::types::EventType;

#[derive(Debug, Clone, Serialize)]
pub struct ReplaySummary {
    pub total_events: usize,
    pub file_events: usize,
    pub command_events: usize,
    pub test_events: usize,
    pub git_events: usize,
    pub agent_messages: usize,
}

pub struct ReplayEngine;

impl ReplayEngine {
    pub fn summarize(events: &[Event]) -> ReplaySummary {
        let mut summary = ReplaySummary {
            total_events: events.len(),
            file_events: 0,
            command_events: 0,
            test_events: 0,
            git_events: 0,
            agent_messages: 0,
        };

        for event in events {
            match event.event_type {
                EventType::FileOpened | EventType::FileModified | EventType::FileCreated => {
                    summary.file_events += 1
                }
                EventType::CommandExecuted => summary.command_events += 1,
                EventType::TestRun | EventType::TestFailed | EventType::TestPassed => {
                    summary.test_events += 1
                }
                EventType::GitCommit => summary.git_events += 1,
                EventType::AgentMessage => summary.agent_messages += 1,
            }
        }

        summary
    }
}
