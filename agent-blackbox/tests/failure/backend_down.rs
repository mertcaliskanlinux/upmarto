use upmarto_sdk::{derive_project_id, resolve_session, EventType, TrackEvent, Upmarto};
use serde_json::json;

use super::common::{
    assert_queue_valid_jsonl, queue_snapshot, read_queue_file, stderr_str, stdout_str, run_cli,
    write_queue_file, sample_event_line, Workspace, DEAD_BACKEND_URL,
};

#[tokio::test]
async fn sdk_flush_returns_err_when_backend_down() {
    let ws = Workspace::new();
    let session = resolve_session(&ws.path);
    let client = Upmarto::from_workspace(&ws.path).await.expect("client");
    client.session(&session).await;

    for i in 0..3 {
        client
            .track(TrackEvent {
                event_type: EventType::FileModified,
                payload: json!({ "path": format!("src/down_{i}.rs") }),
                timestamp: Some(1_700_000_001_000 + i as i64),
            })
            .await
            .expect("track enqueues locally");
    }

    let before = queue_snapshot(&ws.path);
    assert_eq!(read_queue_file(&ws.path).len(), 3);
    assert_queue_valid_jsonl(&read_queue_file(&ws.path));

    let err = client.flush().await.expect_err("flush must fail");
    let msg = err.to_string().to_lowercase();
    assert!(
        msg.contains("unreachable"),
        "expected unreachable in error, got: {msg}"
    );
    assert!(
        msg.contains(DEAD_BACKEND_URL),
        "expected api url in error, got: {msg}"
    );

    let after = queue_snapshot(&ws.path);
    assert_eq!(
        before, after,
        "queue.jsonl must remain intact when backend is down"
    );
    assert_queue_valid_jsonl(&read_queue_file(&ws.path));
    assert_eq!(read_queue_file(&ws.path).len(), 3);
}

#[test]
fn cli_flush_fails_with_stderr_and_exit_code_when_backend_down() {
    let ws = Workspace::new();
    let session = resolve_session(&ws.path);
    let project = derive_project_id(&ws.path);
    let lines: Vec<String> = (0..2)
        .map(|i| sample_event_line(i, &session, &project))
        .collect();
    write_queue_file(&ws.path, &lines);

    let before = queue_snapshot(&ws.path);
    let output = run_cli(&ws.path, &["flush"]);
    let stderr = stderr_str(&output);
    let stdout = stdout_str(&output);

    assert!(
        !output.status.success(),
        "CLI must exit non-zero, stdout={stdout} stderr={stderr}"
    );
    assert_eq!(
        output.status.code(),
        Some(1),
        "expected exit code 1, stderr={stderr}"
    );
    assert!(
        stderr.contains("Error:") && stderr.to_lowercase().contains("unreachable"),
        "expected human-readable stderr, got: {stderr}"
    );
    assert!(
        stdout.is_empty() || !stdout.contains("Successfully flushed"),
        "must not report success on failure, stdout={stdout}"
    );

    let after = queue_snapshot(&ws.path);
    assert_eq!(before, after, "queue.jsonl must not be truncated on failure");
    assert_queue_valid_jsonl(&read_queue_file(&ws.path));
}
