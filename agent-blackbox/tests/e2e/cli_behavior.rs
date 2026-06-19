use serde_json::json;
use upmarto_sdk::{derive_project_id, resolve_session, write_active_session, EventType, TrackEvent, Upmarto};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use crate::reliability::{
    assert_queue_valid_jsonl, read_queue_file, run_cli, sample_event_line, stderr_str, stdout_str,
    write_queue_file, Workspace,
};

fn mock_explain_body() -> serde_json::Value {
    json!({
        "api_version": "v1",
        "explain_schema_version": "v1",
        "summary": "auth_session_expiry failed then passed",
        "root_cause": "auth_session_expiry: session token not refreshed",
        "decision_chain": ["test_failed", "file_modified", "test_passed"],
        "problem_statement": "auth session expiry",
        "resolution_flow": "fix auth.rs and re-run test"
    })
}

#[tokio::test]
async fn cli_explain_without_session_uses_active_session() {
    let server = MockServer::start().await;
    let active = "wf-cli-active-session";

    Mock::given(method("POST"))
        .and(path("/explain"))
        .respond_with(ResponseTemplate::new(200).set_body_json(mock_explain_body()))
        .expect(1)
        .mount(&server)
        .await;

    let ws = Workspace::with_api_url(&server.uri(), 1);
    write_active_session(&ws.path, active).expect("write active session");

    let output = run_cli(&ws.path, &["explain"]);
    let stdout = stdout_str(&output);
    let stderr = stderr_str(&output);

    assert!(
        output.status.success(),
        "explain must succeed without session arg, stderr={stderr}"
    );
    assert!(
        stdout.contains(active),
        "stdout must show active session id, got: {stdout}"
    );
    assert!(
        stdout.contains("auth_session_expiry"),
        "stdout must include explain root cause, got: {stdout}"
    );
}

#[tokio::test]
async fn cli_init_prints_current_session() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/config"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({ "api_version": "v1" })),
        )
        .mount(&server)
        .await;

    let ws = Workspace::with_api_url(&server.uri(), 1);
    let output = run_cli(&ws.path, &["init", "--api-url", &server.uri()]);
    let stdout = stdout_str(&output);

    assert!(output.status.success(), "init failed: {}", stderr_str(&output));
    assert!(
        stdout.contains("Current Session:"),
        "init must print Current Session, got: {stdout}"
    );
    let expected = resolve_session(&ws.path);
    assert!(
        stdout.contains(&expected),
        "init must print daily session id {expected}, got: {stdout}"
    );
}

#[tokio::test]
async fn cli_flush_success_prints_count_and_exits_zero() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/event"))
        .respond_with(ResponseTemplate::new(201))
        .expect(3)
        .mount(&server)
        .await;

    let ws = Workspace::with_api_url(&server.uri(), 1);
    let session = resolve_session(&ws.path);
    let client = Upmarto::from_workspace(&ws.path).await.expect("client");
    client.session(&session).await;

    for i in 0..3 {
        client
            .track(TrackEvent {
                event_type: EventType::FileModified,
                payload: json!({ "path": format!("src/cli_ok_{i}.rs") }),
                timestamp: Some(1_700_000_020_000 + i as i64),
            })
            .await
            .expect("track enqueues to disk");
    }
    assert_eq!(read_queue_file(&ws.path).len(), 3);
    assert_queue_valid_jsonl(&read_queue_file(&ws.path));

    let output = run_cli(&ws.path, &["flush"]);
    let stdout = stdout_str(&output);
    let stderr = stderr_str(&output);

    assert!(
        output.status.success(),
        "CLI must exit 0 on success, stderr={stderr}"
    );
    assert_eq!(output.status.code(), Some(0));
    assert!(
        stdout.contains("Successfully flushed 3 events"),
        "expected success message, got stdout={stdout}"
    );
    assert!(
        stderr.is_empty() || !stderr.contains("Error:"),
        "stderr must not contain error on success, got: {stderr}"
    );
    assert!(
        read_queue_file(&ws.path).is_empty(),
        "queue file must be cleared after successful flush"
    );
}

#[test]
fn cli_flush_failure_exits_one_and_writes_stderr_only() {
    let ws = Workspace::new();
    let session = resolve_session(&ws.path);
    let project = derive_project_id(&ws.path);
    let lines: Vec<String> = (0..1)
        .map(|i| sample_event_line(i, &session, &project))
        .collect();
    write_queue_file(&ws.path, &lines);

    let output = run_cli(&ws.path, &["flush"]);
    let stdout = stdout_str(&output);
    let stderr = stderr_str(&output);

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(1));
    assert!(
        stderr.contains("Error:") && stderr.to_lowercase().contains("unreachable"),
        "stderr must contain error, got: {stderr}"
    );
    assert!(
        !stdout.contains("Successfully flushed"),
        "stdout must not claim success, got: {stdout}"
    );
    assert_eq!(read_queue_file(&ws.path).len(), 1);
}

#[test]
fn cli_flush_empty_queue_exits_zero_with_notice() {
    let ws = Workspace::with_api_url("http://127.0.0.1:1", 1);
    let output = run_cli(&ws.path, &["flush"]);
    let stdout = stdout_str(&output);

    assert!(output.status.success(), "empty queue is not a failure");
    assert_eq!(output.status.code(), Some(0));
    assert!(
        stdout.contains("No pending events"),
        "expected empty-queue message, got: {stdout}"
    );
}
