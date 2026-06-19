use crate::common::{assert_timeline_contains, snapshot_explain, ScenarioBuilder, TestContext};
use serde_json::json;

#[tokio::test]
async fn feature_development_explain_snapshot() {
    let ctx = TestContext::new("test_feature_development").await;
    let session = ctx.session("session");

    let timestamps = [1_700_000_200_001_i64, 1_700_000_200_002, 1_700_000_200_003];

    let timeline = ScenarioBuilder::new(&ctx.client, &session, timestamps[0])
        .push(
            "file_created",
            json!({ "path": "src/notifications.rs", "purpose": "user notification module" }),
        )
        .push(
            "command_executed",
            json!({ "command": "cargo test notifications" }),
        )
        .push(
            "git_commit",
            json!({ "message": "feat: add user notification module" }),
        )
        .send_all()
        .await;

    assert_timeline_contains(
        &timeline,
        &timestamps,
        &["file_created", "command_executed", "git_commit"],
    );

    let explain = ctx.client.post_explain(&session).await;
    snapshot_explain("feature_development", &explain);
}
