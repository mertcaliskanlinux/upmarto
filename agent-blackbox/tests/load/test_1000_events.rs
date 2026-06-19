use std::time::Instant;

use crate::common::{assert_explain_structure, event, TestContext, GOLDEN_PROJECT};
use serde_json::json;

const EVENT_COUNT: usize = 1000;

#[tokio::test]
async fn load_1000_events_timeline_and_explain() {
    let ctx = TestContext::new("load_1000").await;
    let session = ctx.session("session");
    let base_ts = 1_800_000_000_000_i64;

    let types = [
        "file_modified",
        "test_failed",
        "command_executed",
        "git_commit",
        "test_passed",
    ];

    let ingest_start = Instant::now();
    for i in 0..EVENT_COUNT {
        let event_type = types[i % types.len()];
        let payload = match event_type {
            "file_modified" => json!({ "path": format!("src/file_{i}.rs") }),
            "test_failed" => {
                json!({ "test": format!("test_{}", i % 20), "error": "assertion failed" })
            }
            "command_executed" => json!({ "command": format!("cargo test --test t{i}") }),
            "git_commit" => json!({ "message": format!("fix: iteration {i}") }),
            "test_passed" => json!({ "test": format!("test_{}", i % 20) }),
            _ => json!({}),
        };
        ctx.client
            .post_event(event(&session, base_ts + i as i64, event_type, payload))
            .await;
    }
    let ingest_ms = ingest_start.elapsed().as_millis();
    assert!(
        ingest_ms < 120_000,
        "ingesting {EVENT_COUNT} events took {ingest_ms}ms (limit 120s)"
    );

    let timeline_start = Instant::now();
    let timeline = ctx.client.get_timeline(&session).await;
    let timeline_ms = timeline_start.elapsed().as_millis();
    assert_eq!(timeline.events.len(), EVENT_COUNT);
    assert!(
        timeline_ms < 30_000,
        "timeline query for {EVENT_COUNT} events took {timeline_ms}ms (limit 30s)"
    );

    for i in 1..timeline.events.len() {
        assert!(
            timeline.events[i].timestamp >= timeline.events[i - 1].timestamp,
            "timeline out of order at index {i}"
        );
    }

    let explain_start = Instant::now();
    let explain1 = ctx.client.post_explain(&session).await;
    let explain2 = ctx.client.post_explain(&session).await;
    let explain_ms = explain_start.elapsed().as_millis();
    assert_explain_structure(&explain1);
    assert_eq!(
        serde_json::to_string(&explain1).unwrap(),
        serde_json::to_string(&explain2).unwrap(),
        "explain must be deterministic under load"
    );
    assert!(
        explain_ms < 10_000,
        "explain for {EVENT_COUNT} events took {explain_ms}ms (limit 10s)"
    );

    let session_meta = ctx.client.get_session(&session).await;
    assert_eq!(session_meta.session.event_count, EVENT_COUNT as i64);

    let integrity = ctx.client.get_integrity().await;
    assert_eq!(integrity.status, "ok");
    assert_eq!(integrity.jsonl_line_count, EVENT_COUNT as u64);
    assert_eq!(integrity.sqlite_indexed_count, EVENT_COUNT as u64);
    assert_eq!(integrity.orphan_jsonl_count, 0);
    assert_eq!(integrity.missing_offset_count, 0);
    assert!(integrity.broken_session_ids.is_empty());

    let projects = ctx.client.get_project_sessions(GOLDEN_PROJECT).await;
    assert!(projects.sessions.iter().any(|s| s.id == session));
}
