use crate::common::{assert_timeline_contains, snapshot_explain, ScenarioBuilder, TestContext};
use serde_json::json;

#[tokio::test]
async fn regression_fix_explain_snapshot() {
    let ctx = TestContext::new("test_regression_fix").await;
    let session = ctx.session("session");

    let timestamps = [
        1_700_000_300_001_i64,
        1_700_000_300_002,
        1_700_000_300_003,
        1_700_000_300_004,
    ];

    let timeline = ScenarioBuilder::new(&ctx.client, &session, timestamps[0])
        .push(
            "file_modified",
            json!({ "path": "cache.rs", "change": "optimized eviction policy" }),
        )
        .push(
            "test_failed",
            json!({
                "test": "cache_invalidation",
                "error": "stale entries not purged on TTL expiry"
            }),
        )
        .push(
            "file_modified",
            json!({ "path": "cache.rs", "change": "restore TTL purge on eviction" }),
        )
        .push("test_passed", json!({ "test": "cache_invalidation" }))
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
    snapshot_explain("regression_fix", &explain);
}
