use serde_json::json;
use upmarto_sdk::{resolve_session, EventType, TrackEvent, Upmarto};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use super::common::{assert_queue_valid_jsonl, read_queue_file, Workspace};

#[tokio::test]
async fn partial_success_leaves_failed_events_in_queue() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/event"))
        .respond_with(ResponseTemplate::new(201))
        .up_to_n_times(2)
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/event"))
        .respond_with(ResponseTemplate::new(500).set_body_string("internal error"))
        .mount(&server)
        .await;

    let ws = Workspace::with_api_url(&server.uri(), 1);
    let session = resolve_session(&ws.path);
    let client = Upmarto::from_workspace(&ws.path).await.expect("client");
    client.session(&session).await;

    for i in 0..3 {
        client
            .track(TrackEvent {
                event_type: EventType::FileModified,
                payload: json!({ "path": format!("src/partial_{i}.rs") }),
                timestamp: Some(1_700_000_002_000 + i as i64),
            })
            .await
            .expect("track");
    }

    let err = client.flush().await.expect_err("flush must fail after partial");
    let msg = err.to_string();
    assert!(
        msg.contains("500") || msg.contains("internal error"),
        "expected HTTP failure in error, got: {msg}"
    );
    assert!(
        msg.contains("delivered"),
        "expected delivered count in error, got: {msg}"
    );

    let remaining = read_queue_file(&ws.path);
    assert_eq!(remaining.len(), 1, "one failed event must remain queued");
    assert_queue_valid_jsonl(&remaining);

    server.reset().await;
    Mock::given(method("POST"))
        .and(path("/event"))
        .respond_with(ResponseTemplate::new(201))
        .mount(&server)
        .await;

    let flushed = client.flush().await.expect("retry flush succeeds");
    assert_eq!(flushed, 1);
    assert!(read_queue_file(&ws.path).is_empty());
}

#[tokio::test]
async fn no_silent_truncation_on_mixed_batch() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/event"))
        .respond_with(ResponseTemplate::new(201))
        .up_to_n_times(1)
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/event"))
        .respond_with(ResponseTemplate::new(503))
        .mount(&server)
        .await;

    let ws = Workspace::with_api_url(&server.uri(), 1);
    let session = resolve_session(&ws.path);
    let client = Upmarto::from_workspace(&ws.path).await.expect("client");
    client.session(&session).await;

    for i in 0..2 {
        client
            .track(TrackEvent {
                event_type: EventType::TestFailed,
                payload: json!({ "test": format!("t{i}"), "error": "fail" }),
                timestamp: Some(1_700_000_003_000 + i as i64),
            })
            .await
            .expect("track");
    }

    let lines_before_fail = read_queue_file(&ws.path);
    assert_eq!(lines_before_fail.len(), 2);

    let _ = client.flush().await.expect_err("second event fails");

    let lines_after_fail = read_queue_file(&ws.path);
    assert_queue_valid_jsonl(&lines_after_fail);
    assert!(
        !lines_after_fail.is_empty(),
        "queue must not be silently emptied"
    );
    assert!(
        lines_after_fail.len() <= lines_before_fail.len(),
        "queue must not grow on failure"
    );
}
