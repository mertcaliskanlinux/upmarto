use crate::common::{assert_timeline_contains, event, TestContext, GOLDEN_PROJECT};
use serde_json::json;

#[tokio::test]
async fn post_event_persists_and_returns_created_event() {
    let ctx = TestContext::new("test_post_event").await;
    let session = ctx.session("session");
    let ts = 1_700_000_010_001_i64;
    let body = event(&session, ts, "file_opened", json!({ "path": "src/lib.rs" }));

    let created = ctx.client.post_event(body).await;
    assert_eq!(created.event.session_id, session);
    assert_eq!(created.event.project_id, GOLDEN_PROJECT);
    assert_eq!(created.event.event_type, "file_opened");
    assert_eq!(created.event.timestamp, ts);
    assert_eq!(created.event.payload["path"], "src/lib.rs");
    assert!(!created.event.id.is_empty());
}

#[tokio::test]
async fn timeline_returns_timestamp_ordered_events() {
    let ctx = TestContext::new("test_timeline_order").await;
    let session = ctx.session("session");
    let timestamps = [1_700_000_020_001_i64, 1_700_000_020_002, 1_700_000_020_003];
    let events = [
        event(
            &session,
            timestamps[2],
            "command_executed",
            json!({ "command": "cargo test" }),
        ),
        event(
            &session,
            timestamps[0],
            "file_opened",
            json!({ "path": "src/main.rs" }),
        ),
        event(
            &session,
            timestamps[1],
            "file_modified",
            json!({ "path": "src/main.rs" }),
        ),
    ];

    for body in events {
        ctx.client.post_event(body).await;
    }

    let timeline = ctx.client.get_timeline(&session).await;
    assert_eq!(timeline.session_id, session);
    assert_timeline_contains(
        &timeline,
        &timestamps,
        &["file_opened", "file_modified", "command_executed"],
    );
}

#[tokio::test]
async fn session_metadata_reflects_grouped_events() {
    let ctx = TestContext::new("test_session_meta").await;
    let session = ctx.session("session");
    let ts = [1_700_000_030_001_i64, 1_700_000_030_002];
    ctx.client
        .post_event(event(
            &session,
            ts[0],
            "test_failed",
            json!({ "test": "login_flow" }),
        ))
        .await;
    ctx.client
        .post_event(event(
            &session,
            ts[1],
            "test_passed",
            json!({ "test": "login_flow" }),
        ))
        .await;

    let session_resp = ctx.client.get_session(&session).await;
    assert_eq!(session_resp.session.id, session);
    assert_eq!(session_resp.session.project_id, GOLDEN_PROJECT);
    assert_eq!(session_resp.session.event_count, 2);
    assert_eq!(session_resp.session.started_at, ts[0]);
    assert_eq!(session_resp.session.ended_at, Some(ts[1]));
}

#[tokio::test]
async fn project_sessions_lists_session() {
    let ctx = TestContext::new("test_project_sessions").await;
    let session = ctx.session("session");
    ctx.client
        .post_event(event(
            &session,
            1_700_000_040_001,
            "git_commit",
            json!({ "message": "chore: integration test" }),
        ))
        .await;

    let projects = ctx.client.get_project_sessions(GOLDEN_PROJECT).await;
    assert_eq!(projects.project_id, GOLDEN_PROJECT);
    assert_eq!(projects.sessions.len(), 1);
    assert_eq!(projects.sessions[0].id, session);
}

#[tokio::test]
async fn explain_returns_structured_reasoning() {
    let ctx = TestContext::new("test_explain_structure").await;
    let session = ctx.session("session");
    ctx.client
        .post_event(event(
            &session,
            1_700_000_050_001,
            "test_failed",
            json!({ "test": "api_health", "error": "connection refused" }),
        ))
        .await;
    ctx.client
        .post_event(event(
            &session,
            1_700_000_050_002,
            "file_modified",
            json!({ "path": "src/config.rs" }),
        ))
        .await;
    ctx.client
        .post_event(event(
            &session,
            1_700_000_050_003,
            "test_passed",
            json!({ "test": "api_health" }),
        ))
        .await;

    let explain = ctx.client.post_explain(&session).await;
    assert!(explain.summary.contains("api_health") || explain.summary.contains("fix"));
    assert!(explain.root_cause.contains("api_health"));
    assert!(explain
        .decision_chain
        .iter()
        .any(|step| step.contains("test_failed")));
}
