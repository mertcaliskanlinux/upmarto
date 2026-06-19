use serde_json::json;
use upmarto_sdk::{derive_project_id, resolve_session, EventType, TrackEvent, Upmarto};

use super::common::{
    assert_queue_valid_jsonl, queue_snapshot, read_queue_file, run_cli, sample_event_line,
    write_queue_file, Workspace, DEAD_BACKEND_URL,
};

#[tokio::test]
async fn queue_jsonl_stays_valid_through_failed_flush() {
    let ws = Workspace::new();
    let session = resolve_session(&ws.path);
    let _project = derive_project_id(&ws.path);
    let client = Upmarto::from_workspace(&ws.path).await.expect("client");
    client.session(&session).await;

    for i in 0..4 {
        client
            .track(TrackEvent {
                event_type: EventType::FileModified,
                payload: json!({ "path": format!("src/integrity_{i}.rs") }),
                timestamp: Some(1_700_000_005_000 + i as i64),
            })
            .await
            .expect("track");
    }

    let lines = read_queue_file(&ws.path);
    assert_eq!(lines.len(), 4);
    assert_queue_valid_jsonl(&lines);

    let _ = client.flush().await.expect_err("backend down");

    let after = read_queue_file(&ws.path);
    assert_eq!(after.len(), 4, "no event may be dropped silently");
    assert_queue_valid_jsonl(&after);
    assert_eq!(queue_snapshot(&ws.path), queue_snapshot(&ws.path));
}

#[test]
fn prewritten_queue_survives_cli_failure_unchanged() {
    let ws = Workspace::new();
    let session = resolve_session(&ws.path);
    let project = derive_project_id(&ws.path);
    let lines: Vec<String> = (0..5)
        .map(|i| sample_event_line(i, &session, &project))
        .collect();
    write_queue_file(&ws.path, &lines);

    assert_queue_valid_jsonl(&read_queue_file(&ws.path));
    let snapshot = queue_snapshot(&ws.path);

    let output = run_cli(&ws.path, &["flush"]);
    assert!(!output.status.success());

    let after = read_queue_file(&ws.path);
    assert_eq!(after.len(), 5);
    assert_queue_valid_jsonl(&after);
    assert_eq!(queue_snapshot(&ws.path), snapshot);

    for line in &after {
        assert!(
            !line.contains("undefined") && line.starts_with('{'),
            "queue must not be partially corrupted"
        );
    }
}

#[tokio::test]
async fn events_either_sent_or_retained_never_dropped() {
    let ws = Workspace::new();
    let session = resolve_session(&ws.path);
    let client = Upmarto::from_workspace(&ws.path).await.expect("client");
    client.session(&session).await;

    let event_count = 3usize;
    for i in 0..event_count {
        client
            .track(TrackEvent {
                event_type: EventType::AgentMessage,
                payload: json!({ "message": format!("msg {i}") }),
                timestamp: Some(1_700_000_006_000 + i as i64),
            })
            .await
            .expect("track");
    }

    assert_eq!(read_queue_file(&ws.path).len(), event_count);
    let err = client.flush().await.expect_err("unreachable backend");
    assert!(
        err.to_string().contains(DEAD_BACKEND_URL),
        "error must surface backend url"
    );
    assert_eq!(
        read_queue_file(&ws.path).len(),
        event_count,
        "all events must be retained for retry"
    );
}
