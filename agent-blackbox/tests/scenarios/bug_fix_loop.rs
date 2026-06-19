use crate::common::{assert_timeline_contains, snapshot_explain, ScenarioBuilder, TestContext};
use serde_json::json;

#[tokio::test]
async fn bug_fix_loop_explain_snapshot() {
    let ctx = TestContext::new("test_bug_fix_loop").await;
    let session = ctx.session("session");

    let timestamps = [
        1_700_000_100_001_i64,
        1_700_000_100_002,
        1_700_000_100_003,
        1_700_000_100_004,
    ];

    let timeline = ScenarioBuilder::new(&ctx.client, &session, timestamps[0])
        .push(
            "file_modified",
            json!({ "path": "auth.go", "change": "initial session handling" }),
        )
        .push(
            "test_failed",
            json!({
                "test": "auth_session_expiry",
                "error": "session token not refreshed after expiry"
            }),
        )
        .push(
            "file_modified",
            json!({ "path": "auth.go", "change": "added cache strategy for expired sessions" }),
        )
        .push("test_passed", json!({ "test": "auth_session_expiry" }))
        .send_all()
        .await;

    assert_timeline_contains(
        &timeline,
        &timestamps,
        &[
            "file_modified",
            "test_failed",
            "file_modified",
            "test_passed",
        ],
    );

    let explain = ctx.client.post_explain(&session).await;
    snapshot_explain("bug_fix_loop", &explain);
}
