use std::path::PathBuf;
use std::process;

use clap::{Parser, Subcommand};
use upmarto_sdk::{
    bootstrap_workspace, derive_project_id, new_workflow_session_id, read_active_session,
    resolve_session, write_active_session, EventType, TrackEvent, Upmarto, UpmartoConfig,
};

#[derive(Parser)]
#[command(name = "upmarto", about = "Upmarto CLI — AI agent memory & reasoning")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Workspace root (defaults to current directory)
    #[arg(long, global = true)]
    workspace: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
    /// Create `.upmarto/config.json` in the project
    Init {
        #[arg(long)]
        api_url: Option<String>,
    },
    /// Track a single event
    Track {
        /// Event type (e.g. file_modified, test_failed)
        #[arg(long, short)]
        r#type: String,
        /// JSON payload (PowerShell: use --path/--test flags instead)
        #[arg(long, default_value = "{}")]
        payload: String,
        /// Read JSON payload from file
        #[arg(long)]
        payload_file: Option<PathBuf>,
        /// Shorthand: file path (file_opened, file_modified, file_created)
        #[arg(long)]
        path: Option<String>,
        /// Shorthand: test name (test_run, test_failed, test_passed)
        #[arg(long)]
        test: Option<String>,
        /// Shorthand: error message (test_failed)
        #[arg(long)]
        error: Option<String>,
        /// Shorthand: shell command (command_executed)
        #[arg(long)]
        command: Option<String>,
        /// Shorthand: message (git_commit, agent_message)
        #[arg(long)]
        message: Option<String>,
    },
    /// Send a demo bug-fix scenario (4 events) for quick testing
    Demo,
    /// Run an isolated bug-fix workflow (6 events) for clean explain output
    Workflow,
    /// Show current session id for this workspace
    Session,
    /// Explain a session (WHY engine). Uses active session when omitted.
    Explain {
        /// Session id (optional — defaults to active CLI session)
        session_id: Option<String>,
    },
    /// Flush pending event queue
    Flush,
}

#[tokio::main]
async fn main() {
    if let Err(err) = run().await {
        eprintln!("Error: {err}");
        process::exit(1);
    }
}

async fn run() -> upmarto_sdk::Result<()> {
    let cli = Cli::parse();
    let workspace = cli
        .workspace
        .unwrap_or_else(|| std::env::current_dir().expect("cwd"));

    match cli.command {
        Commands::Init { api_url } => cmd_init(&workspace, api_url).await,
        Commands::Track {
            r#type,
            payload,
            payload_file,
            path,
            test,
            error,
            command,
            message,
        } => {
            cmd_track(
                &workspace,
                &r#type,
                &payload,
                payload_file.as_deref(),
                path.as_deref(),
                test.as_deref(),
                error.as_deref(),
                command.as_deref(),
                message.as_deref(),
            )
            .await
        }
        Commands::Demo => cmd_demo(&workspace).await,
        Commands::Workflow => cmd_workflow(&workspace).await,
        Commands::Session => cmd_session(&workspace),
        Commands::Explain { session_id } => cmd_explain(&workspace, session_id.as_deref()).await,
        Commands::Flush => cmd_flush(&workspace).await,
    }
}

fn resolve_cli_session(workspace: &PathBuf) -> String {
    read_active_session(workspace).unwrap_or_else(|| resolve_session(workspace))
}

fn print_current_session(session_id: &str) {
    println!("Current Session:");
    println!("  {session_id}");
}

async fn cmd_init(workspace: &PathBuf, api_url: Option<String>) -> upmarto_sdk::Result<()> {
    let cfg = UpmartoConfig {
        api_url: String::new(),
        project_id: "auto".into(),
        auto_capture: true,
        batch_size: 50,
        flush_interval_ms: 2000,
        retry_max: 5,
    };

    let url = bootstrap_workspace(workspace, api_url, &cfg).await?;

    let session_id = resolve_session(workspace);
    write_active_session(workspace, &session_id)?;

    println!("✓ Upmarto initialized");
    println!("  api_url: {url}");
    println!("  runtime: .upmarto/runtime.json");
    println!("  project_id: auto (→ {})", derive_project_id(workspace));
    print_current_session(&session_id);
    println!("  run: upmarto explain");
    Ok(())
}

async fn cmd_track(
    workspace: &PathBuf,
    event_type: &str,
    payload: &str,
    payload_file: Option<&std::path::Path>,
    path: Option<&str>,
    test: Option<&str>,
    error: Option<&str>,
    command: Option<&str>,
    message: Option<&str>,
) -> upmarto_sdk::Result<()> {
    let client = Upmarto::from_workspace(workspace).await?;
    let session_id = resolve_cli_session(workspace);
    client.session(session_id).await;

    let event_type = parse_event_type(event_type)?;
    let payload = build_payload(payload, payload_file, path, test, error, command, message)?;

    client
        .track(TrackEvent {
            event_type,
            payload,
            timestamp: None,
        })
        .await?;
    let flushed = client.flush().await?;
    if flushed == 0 {
        return Err(upmarto_sdk::SdkError::Api(
            "Event was not delivered — Upmarto backend unreachable. Run: upmarto init".into(),
        ));
    }
    println!("✓ Event tracked");
    Ok(())
}

fn build_payload(
    payload: &str,
    payload_file: Option<&std::path::Path>,
    path: Option<&str>,
    test: Option<&str>,
    error: Option<&str>,
    command: Option<&str>,
    message: Option<&str>,
) -> upmarto_sdk::Result<serde_json::Value> {
    let mut value: serde_json::Map<String, serde_json::Value> = if let Some(file) = payload_file {
        let raw = std::fs::read_to_string(file)?;
        serde_json::from_str(&raw)?
    } else if payload.trim() == "{}" {
        serde_json::Map::new()
    } else {
        serde_json::from_str(payload)?
    };

    if let Some(p) = path {
        value.insert("path".into(), serde_json::Value::String(p.into()));
    }
    if let Some(t) = test {
        value.insert("test".into(), serde_json::Value::String(t.into()));
    }
    if let Some(e) = error {
        value.insert("error".into(), serde_json::Value::String(e.into()));
    }
    if let Some(c) = command {
        value.insert("command".into(), serde_json::Value::String(c.into()));
    }
    if let Some(m) = message {
        value.insert("message".into(), serde_json::Value::String(m.into()));
    }

    Ok(serde_json::Value::Object(value))
}

async fn track_events(
    client: &Upmarto,
    events: &[(EventType, serde_json::Value)],
) -> upmarto_sdk::Result<usize> {
    for (event_type, payload) in events {
        client
            .track(TrackEvent {
                event_type: event_type.clone(),
                payload: payload.clone(),
                timestamp: None,
            })
            .await?;
    }
    client.flush().await
}

async fn cmd_demo(workspace: &PathBuf) -> upmarto_sdk::Result<()> {
    let client = Upmarto::from_workspace(workspace).await?;
    let session_id = new_workflow_session_id();
    write_active_session(workspace, &session_id)?;
    client.session(session_id.clone()).await;

    let events = [
        (
            EventType::FileModified,
            serde_json::json!({ "path": "src/main.rs" }),
        ),
        (
            EventType::TestFailed,
            serde_json::json!({ "test": "api_health", "error": "connection refused" }),
        ),
        (
            EventType::FileModified,
            serde_json::json!({ "path": "src/config.rs" }),
        ),
        (
            EventType::TestPassed,
            serde_json::json!({ "test": "api_health" }),
        ),
    ];

    let flushed = track_events(&client, &events).await?;

    println!("✓ Demo scenario tracked ({flushed} events)");
    print_current_session(&session_id);
    println!("  run: upmarto explain");
    Ok(())
}

async fn cmd_workflow(workspace: &PathBuf) -> upmarto_sdk::Result<()> {
    let client = Upmarto::from_workspace(workspace).await?;
    let session_id = new_workflow_session_id();
    write_active_session(workspace, &session_id)?;
    client.session(session_id.clone()).await;

    let events = [
        (
            EventType::FileOpened,
            serde_json::json!({ "path": "src/main.rs", "source": "cli" }),
        ),
        (
            EventType::FileModified,
            serde_json::json!({ "path": "src/auth.rs", "source": "cli" }),
        ),
        (
            EventType::TestFailed,
            serde_json::json!({
                "test": "auth_session_expiry",
                "error": "session token not refreshed",
                "source": "cli"
            }),
        ),
        (
            EventType::FileModified,
            serde_json::json!({ "path": "src/auth.rs", "source": "cli", "change": "fix" }),
        ),
        (
            EventType::TestPassed,
            serde_json::json!({ "test": "auth_session_expiry", "source": "cli" }),
        ),
        (
            EventType::GitCommit,
            serde_json::json!({
                "message": "fix: auth session expiry handling",
                "source": "cli"
            }),
        ),
    ];

    let flushed = track_events(&client, &events).await?;

    println!("✓ Workflow tracked ({flushed} events)");
    print_current_session(&session_id);
    println!("  run: upmarto explain");
    Ok(())
}

fn cmd_session(workspace: &PathBuf) -> upmarto_sdk::Result<()> {
    let session_id = resolve_cli_session(workspace);
    let project_id = derive_project_id(workspace);
    println!("project_id: {project_id}");
    print_current_session(&session_id);
    Ok(())
}

async fn cmd_explain(workspace: &PathBuf, session_id: Option<&str>) -> upmarto_sdk::Result<()> {
    let session_id = session_id
        .map(str::to_string)
        .unwrap_or_else(|| resolve_cli_session(workspace));

    let client = Upmarto::from_workspace(workspace).await?;
    println!("Explaining session:");
    println!("  {session_id}");
    println!();

    let explain = client.explain(&session_id).await?;

    println!("── Summary ──");
    println!("{}", explain.summary);
    println!();
    println!("── Root cause ──");
    println!("{}", explain.root_cause);
    println!();
    println!("── Problem ──");
    println!("{}", explain.problem_statement);
    println!();
    println!("── Resolution ──");
    println!("{}", explain.resolution_flow);
    println!();
    println!("── Decision chain ──");
    for (i, step) in explain.decision_chain.iter().enumerate() {
        println!("  {}. {}", i + 1, step);
    }
    Ok(())
}

async fn cmd_flush(workspace: &PathBuf) -> upmarto_sdk::Result<()> {
    let client = Upmarto::from_workspace(workspace).await?;
    let restored = client.restore_persisted_queue().await?;
    let flushed = client.flush().await?;

    if flushed == 0 && restored == 0 {
        println!("No pending events to flush.");
    } else {
        println!("Successfully flushed {flushed} events.");
    }
    Ok(())
}

fn parse_event_type(s: &str) -> upmarto_sdk::Result<EventType> {
    match s {
        "file_opened" => Ok(EventType::FileOpened),
        "file_modified" => Ok(EventType::FileModified),
        "file_created" => Ok(EventType::FileCreated),
        "command_executed" => Ok(EventType::CommandExecuted),
        "test_run" => Ok(EventType::TestRun),
        "test_failed" => Ok(EventType::TestFailed),
        "test_passed" => Ok(EventType::TestPassed),
        "git_commit" => Ok(EventType::GitCommit),
        "agent_message" => Ok(EventType::AgentMessage),
        other => Err(upmarto_sdk::SdkError::Config(format!(
            "unknown event type: {other}"
        ))),
    }
}
