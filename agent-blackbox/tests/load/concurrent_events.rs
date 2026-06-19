use std::sync::Arc;
use std::time::Instant;

use crate::common::{event, TestContext, GOLDEN_PROJECT};
use serde_json::json;
use tokio::task::JoinSet;

const WORKERS: usize = 10;
const EVENTS_PER_WORKER: usize = 50;

#[tokio::test]
async fn concurrent_event_ingest_no_loss() {
    let ctx = Arc::new(TestContext::new("load_concurrent").await);
    let session = ctx.session("session");
    let base_ts = 1_900_000_000_000_i64;
    let expected_total = WORKERS * EVENTS_PER_WORKER;

    let start = Instant::now();
    let mut set = JoinSet::new();

    for worker in 0..WORKERS {
        let ctx = Arc::clone(&ctx);
        let session = session.clone();
        set.spawn(async move {
            for i in 0..EVENTS_PER_WORKER {
                let seq = worker * EVENTS_PER_WORKER + i;
                ctx.client
                    .post_event(event(
                        &session,
                        base_ts + seq as i64,
                        if i % 2 == 0 {
                            "file_modified"
                        } else {
                            "command_executed"
                        },
                        if i % 2 == 0 {
                            json!({ "path": format!("w{worker}_f{i}.rs") })
                        } else {
                            json!({ "command": format!("echo worker-{worker}-{i}") })
                        },
                    ))
                    .await;
            }
        });
    }

    while set.join_next().await.is_some() {}
    let ingest_ms = start.elapsed().as_millis();
    assert!(
        ingest_ms < 60_000,
        "concurrent ingest took {ingest_ms}ms (limit 60s)"
    );

    let timeline = ctx.client.get_timeline(&session).await;
    assert_eq!(
        timeline.events.len(),
        expected_total,
        "event loss detected: expected {expected_total}, got {}",
        timeline.events.len()
    );

    for i in 1..timeline.events.len() {
        assert!(
            timeline.events[i].timestamp >= timeline.events[i - 1].timestamp,
            "timeline out of order at index {i}"
        );
    }

    let session_meta = ctx.client.get_session(&session).await;
    assert_eq!(session_meta.session.event_count, expected_total as i64);

    let integrity = ctx.client.get_integrity().await;
    assert_eq!(integrity.status, "ok");
    assert_eq!(integrity.jsonl_line_count, expected_total as u64);
    assert_eq!(integrity.sqlite_indexed_count, expected_total as u64);
    assert_eq!(integrity.orphan_jsonl_count, 0);
    assert_eq!(integrity.missing_offset_count, 0);
    assert!(integrity.broken_session_ids.is_empty());

    let projects = ctx.client.get_project_sessions(GOLDEN_PROJECT).await;
    assert!(projects.sessions.iter().any(|s| s.id == session));
}
