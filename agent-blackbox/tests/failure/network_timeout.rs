use std::time::Duration;

use serde_json::json;
use upmarto_sdk::{resolve_session, EventType, TrackEvent, Upmarto};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use super::common::{assert_queue_valid_jsonl, queue_snapshot, read_queue_file, Workspace};

#[tokio::test]
async fn sdk_flush_returns_err_on_request_timeout() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/event"))
        .respond_with(
            ResponseTemplate::new(201).set_delay(Duration::from_secs(15)),
        )
        .mount(&server)
        .await;

    let ws = Workspace::with_api_url(&server.uri(), 1);
    let session = resolve_session(&ws.path);
    let client = Upmarto::from_workspace(&ws.path).await.expect("client");
    client.session(&session).await;

    client
        .track(TrackEvent {
            event_type: EventType::CommandExecuted,
            payload: json!({ "command": "cargo test" }),
            timestamp: Some(1_700_000_004_000),
        })
        .await
        .expect("track");

    let before = queue_snapshot(&ws.path);
    let err = client.flush().await.expect_err("flush must fail on timeout");
    let msg = err.to_string().to_lowercase();
    assert!(
        msg.contains("timed out") || msg.contains("timeout"),
        "expected timeout-specific error, got: {msg}"
    );

    let after = queue_snapshot(&ws.path);
    assert_eq!(before, after, "queue must be preserved after timeout");
    assert_eq!(read_queue_file(&ws.path).len(), 1);
    assert_queue_valid_jsonl(&read_queue_file(&ws.path));
}
