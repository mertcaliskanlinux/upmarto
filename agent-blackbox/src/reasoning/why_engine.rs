use crate::core::event::Event;
use crate::core::types::EventType;

use super::analyzer::{
    agent_text, analyze, command_text, commit_message, error_detail, file_path, test_name,
    SessionAnalysis, TriggerKind,
};
use super::patterns::PatternKind;

#[derive(Debug, Clone, serde::Serialize)]
pub struct Explanation {
    pub summary: String,
    pub root_cause: String,
    pub decision_chain: Vec<String>,
    pub problem_statement: String,
    pub resolution_flow: String,
}

pub struct WhyEngine;

impl WhyEngine {
    pub fn explain(events: &[Event]) -> Explanation {
        if events.is_empty() {
            return Explanation {
                summary: "No events recorded for this session.".to_string(),
                root_cause: "Insufficient event data.".to_string(),
                decision_chain: vec![],
                problem_statement: "Unable to determine — session has no recorded activity."
                    .to_string(),
                resolution_flow: "No actions were captured.".to_string(),
            };
        }

        let analysis = analyze(events);
        let decision_chain = build_decision_chain(events, &analysis);
        let problem_statement = build_problem_statement(&analysis);
        let root_cause = build_root_cause(events, &analysis);
        let resolution_flow = build_resolution_flow(events, &analysis);
        let summary = build_summary(&analysis, &problem_statement, &resolution_flow);

        Explanation {
            summary,
            root_cause,
            decision_chain,
            problem_statement,
            resolution_flow,
        }
    }
}

fn build_problem_statement(analysis: &SessionAnalysis) -> String {
    if let Some(trigger) = &analysis.primary_trigger {
        return match trigger {
            TriggerKind::TestFailure(test) => {
                format!("Resolve failing test: {test}")
            }
            TriggerKind::TestPassed(test) => {
                format!("Verify and maintain passing test: {test}")
            }
            TriggerKind::GitCommit(msg) => {
                format!("Complete work for commit: {}", truncate(msg, 120))
            }
            TriggerKind::Command(cmd) => {
                format!("Complete task initiated by command: {cmd}")
            }
            TriggerKind::AgentDirective(text) => {
                format!("Execute agent directive: {}", truncate(text, 120))
            }
            TriggerKind::FileChange(path) => {
                format!("Implement or update functionality in {path}")
            }
        };
    }

    if !analysis.affected_files.is_empty() {
        return format!(
            "Modify or extend code across: {}",
            analysis.affected_files.join(", ")
        );
    }

    "Complete the coding task in this session".to_string()
}

fn build_root_cause(events: &[Event], analysis: &SessionAnalysis) -> String {
    if let Some(failure) = events
        .iter()
        .find(|e| e.event_type == EventType::TestFailed)
    {
        let test = test_name(failure).unwrap_or_else(|| "test suite".to_string());
        let detail = error_detail(failure)
            .map(|d| format!(" — {d}"))
            .unwrap_or_default();
        return format!("Test failure in {test}{detail}");
    }

    if let Some(msg) = analysis.agent_messages.first() {
        return format!("Agent directive: {}", truncate(msg, 150));
    }

    if let Some(path) = analysis.affected_files.first() {
        return format!("Required code changes in {path}");
    }

    if let Some(cmd) = analysis.commands.first() {
        return format!("Task driven by command execution: {cmd}");
    }

    "Session activity with no explicit failure signal".to_string()
}

fn build_decision_chain(events: &[Event], analysis: &SessionAnalysis) -> Vec<String> {
    let mut chain = Vec::new();

    for pattern in &analysis.patterns {
        chain.push(format!(
            "pattern detected → {} → causal arc identified",
            pattern.description
        ));
    }

    for (i, event) in events.iter().enumerate() {
        if let Some(link) = interpret_event(event, i, events) {
            chain.push(link);
        }
    }

    if chain.is_empty() {
        chain.push("session_start → agent activity → session captured".to_string());
    }

    chain
}

fn interpret_event(event: &Event, idx: usize, all: &[Event]) -> Option<String> {
    let prev = if idx > 0 { Some(&all[idx - 1]) } else { None };

    let interpretation = match event.event_type {
        EventType::TestFailed => {
            let test = test_name(event).unwrap_or_else(|| "test".to_string());
            format!("test_failed ({test}) → regression or bug detected → motivates investigation")
        }
        EventType::TestPassed => {
            let test = test_name(event).unwrap_or_else(|| "test".to_string());
            let had_failure = all[..idx]
                .iter()
                .any(|e| e.event_type == EventType::TestFailed);
            if had_failure {
                format!("test_passed ({test}) → fix validated → problem resolved")
            } else {
                format!("test_passed ({test}) → change verified → confidence increased")
            }
        }
        EventType::TestRun => {
            "test_run → agent validating current state before proceeding".to_string()
        }
        EventType::FileOpened => {
            let path = file_path(event).unwrap_or_else(|| "file".to_string());
            let reason = match prev.map(|p| p.event_type) {
                Some(EventType::TestFailed) => "inspect code related to failing test",
                Some(EventType::AgentMessage) => "follow up on agent reasoning",
                _ => "understand existing implementation",
            };
            format!("file_opened ({path}) → {reason}")
        }
        EventType::FileModified => {
            let path = file_path(event).unwrap_or_else(|| "file".to_string());
            let reason = match prev.map(|p| p.event_type) {
                Some(EventType::TestFailed) | Some(EventType::FileOpened) => {
                    "apply fix or implement solution"
                }
                Some(EventType::AgentMessage) => "implement planned change",
                _ => "update implementation",
            };
            format!("file_modified ({path}) → {reason}")
        }
        EventType::FileCreated => {
            let path = file_path(event).unwrap_or_else(|| "file".to_string());
            format!("file_created ({path}) → introduce new module or artifact")
        }
        EventType::CommandExecuted => {
            let cmd = command_text(event).unwrap_or_else(|| "command".to_string());
            let reason = if cmd.contains("test") {
                "run tests to verify changes"
            } else if cmd.contains("git") {
                "inspect or manage version control state"
            } else {
                "execute tooling to advance the task"
            };
            format!("command_executed ({cmd}) → {reason}")
        }
        EventType::GitCommit => {
            let msg = commit_message(event)
                .map(|m| truncate(&m, 60))
                .unwrap_or_else(|| "changes".to_string());
            format!("git_commit ({msg}) → finalize and persist completed work")
        }
        EventType::AgentMessage => {
            let text = agent_text(event)
                .map(|t| truncate(&t, 80))
                .unwrap_or_else(|| "reasoning".to_string());
            format!("agent_message ({text}) → sets intent and guides next actions")
        }
    };

    Some(interpretation)
}

fn build_resolution_flow(events: &[Event], analysis: &SessionAnalysis) -> String {
    let mut steps: Vec<String> = Vec::new();
    let mut step_num = 1;

    if let Some(failure) = events
        .iter()
        .find(|e| e.event_type == EventType::TestFailed)
    {
        let test = test_name(failure).unwrap_or_else(|| "test".to_string());
        steps.push(format!("{step_num}. Detected failure in {test}"));
        step_num += 1;
    } else if let Some(msg) = analysis.agent_messages.first() {
        steps.push(format!(
            "{step_num}. Received agent directive: {}",
            truncate(msg, 80)
        ));
        step_num += 1;
    }

    let mut modified: Vec<String> = events
        .iter()
        .filter(|e| {
            matches!(
                e.event_type,
                EventType::FileModified | EventType::FileCreated
            )
        })
        .filter_map(file_path)
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    modified.sort();

    if !modified.is_empty() {
        steps.push(format!("{step_num}. Modified {}", modified.join(", ")));
        step_num += 1;
    }

    let test_commands: Vec<&str> = analysis
        .commands
        .iter()
        .filter(|c| c.contains("test"))
        .map(|s| s.as_str())
        .collect();

    if !test_commands.is_empty() {
        steps.push(format!(
            "{step_num}. Ran verification: {}",
            test_commands.join(", ")
        ));
        step_num += 1;
    } else if analysis.commands.len() == 1 {
        steps.push(format!("{step_num}. Executed: {}", analysis.commands[0]));
        step_num += 1;
    }

    if !analysis.passed_tests.is_empty() {
        steps.push(format!(
            "{step_num}. Validated fix — {} passed",
            analysis.passed_tests.join(", ")
        ));
        step_num += 1;
    }

    if !analysis.commit_messages.is_empty() {
        steps.push(format!(
            "{step_num}. Committed: {}",
            truncate(&analysis.commit_messages[0], 80)
        ));
    } else if has_pattern_kind(analysis, |k| {
        matches!(k, PatternKind::CommitAfterChanges { .. })
    }) {
        steps.push(format!("{step_num}. Committed changes to version control"));
    }

    if steps.is_empty() {
        return describe_generic_flow(events);
    }

    steps.join(". ") + "."
}

fn describe_generic_flow(events: &[Event]) -> String {
    let actions: Vec<String> = events
        .iter()
        .map(|e| e.event_type.as_str().replace('_', " "))
        .collect();

    if actions.is_empty() {
        "No resolution steps recorded.".to_string()
    } else {
        format!("Agent progressed through: {}.", actions.join(" → "))
    }
}

fn build_summary(analysis: &SessionAnalysis, problem: &str, resolution: &str) -> String {
    if has_pattern_kind(analysis, |k| matches!(k, PatternKind::TestFixCycle { .. })) {
        let test = analysis
            .failed_tests
            .first()
            .cloned()
            .unwrap_or_else(|| "test".to_string());
        let files = if analysis.affected_files.is_empty() {
            "relevant source files".to_string()
        } else {
            analysis.affected_files.join(", ")
        };
        let validated = if !analysis.passed_tests.is_empty() {
            format!("validated via {}", analysis.passed_tests.join(", "))
        } else {
            "validated via test success".to_string()
        };

        return format!(
            "Agent attempted to fix a failure in {test}. It investigated and modified {files}, then {validated}."
        );
    }

    if !analysis.agent_messages.is_empty() && !analysis.affected_files.is_empty() {
        return format!(
            "Agent pursued: {problem}. It worked on {} and {}",
            analysis.affected_files.join(", "),
            truncate(resolution.trim_end_matches('.'), 120)
        );
    }

    if !analysis.affected_files.is_empty() && !analysis.commands.is_empty() {
        return format!(
            "Agent modified {} and ran {} to advance the task.",
            analysis.affected_files.join(", "),
            analysis.commands.join(", ")
        );
    }

    format!(
        "Agent worked on: {problem}. {}",
        truncate(resolution.trim_end_matches('.'), 200)
    )
}

fn has_pattern_kind<F>(analysis: &SessionAnalysis, predicate: F) -> bool
where
    F: Fn(&PatternKind) -> bool,
{
    analysis.patterns.iter().any(|p| predicate(&p.kind))
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        format!("{}…", s.chars().take(max).collect::<String>())
    }
}
