use serde_json::json;
use upmarto_sdk::{derive_project_id, resolve_session, EventType, TrackEvent, Upmarto};

use crate::common::TestContext;
use crate::reliability::{
    assert_queue_valid_jsonl, read_queue_file, write_workspace_config, Workspace,
};

#[tokio::test]
async fn queued_events_replay_after_backend_recovery() {
    let ws = Workspace::new();
    let session = resolve_session(&ws.path);
    let project = derive_project_id(&ws.path);

    let client = Upmarto::from_workspace(&ws.path).await.expect("client");
    client.session(&session).await;

    let event_count = 4usize;
    for i in 0..event_count {
        client
            .track(TrackEvent {
                event_type: EventType::FileModified,
                payload: json!({ "path": format!("src/recovery_{i}.rs") }),
                timestamp: Some(1_700_000_010_000 + i as i64),
            })
            .await
            .expect("track while backend down");
    }

    assert_eq!(read_queue_file(&ws.path).len(), event_count);
    assert!(
        client.flush().await.is_err(),
        "flush must fail while backend is down"
    );
    assert_eq!(
        read_queue_file(&ws.path).len(),
        event_count,
        "queue preserved during outage"
    );

    let ctx = TestContext::new("e2e_recovery").await;
    write_workspace_config(&ws.path, ctx.client.base_url(), 1);

    let client = Upmarto::from_workspace(&ws.path).await.expect("reloaded client");
    client.session(&session).await;

    let restored = client
        .restore_persisted_queue()
        .await
        .expect("restore queue");
    assert_eq!(restored, event_count);

    let flushed = client.flush().await.expect("flush after recovery");
    assert_eq!(flushed, event_count);
    assert!(read_queue_file(&ws.path).is_empty());
    assert_queue_valid_jsonl(&read_queue_file(&ws.path));

    let timeline = ctx.client.get_timeline(&session).await;
    assert_eq!(
        timeline.events.len(),
        event_count,
        "timeline must contain full recovered event set"
    );

    let paths: Vec<String> = timeline
        .events
        .iter()
        .filter_map(|e| e.payload.get("path").and_then(|p| p.as_str().map(str::to_string)))
        .collect();
    for i in 0..event_count {
        let expected = format!("src/recovery_{i}.rs");
        assert!(
            paths.contains(&expected),
            "missing event path {expected} in timeline"
        );
    }

    // No duplicate delivery beyond single successful ingest per queued event
    assert_eq!(
        paths.len(),
        event_count,
        "unexpected duplicate events in timeline"
    );

    // Events belong to correct project
    assert!(
        timeline
            .events
            .iter()
            .all(|e| e.project_id == project),
        "project_id must match workspace"
    );
}
