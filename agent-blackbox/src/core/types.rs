use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    FileOpened,
    FileModified,
    FileCreated,
    CommandExecuted,
    TestRun,
    TestFailed,
    TestPassed,
    GitCommit,
    AgentMessage,
}

impl EventType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::FileOpened => "file_opened",
            Self::FileModified => "file_modified",
            Self::FileCreated => "file_created",
            Self::CommandExecuted => "command_executed",
            Self::TestRun => "test_run",
            Self::TestFailed => "test_failed",
            Self::TestPassed => "test_passed",
            Self::GitCommit => "git_commit",
            Self::AgentMessage => "agent_message",
        }
    }
}
