mod server;

use std::time::Duration;

use reqwest::{Client as HttpClient, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

pub use server::TestContext;

pub const GOLDEN_PROJECT: &str = "golden-tests";

#[derive(Clone)]
pub struct Client {
    http: HttpClient,
    base_url: String,
}

impl Client {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            http: HttpClient::builder()
                .timeout(Duration::from_secs(120))
                .build()
                .expect("failed to build HTTP client"),
            base_url: base_url.into(),
        }
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub async fn post_event(&self, body: Value) -> EventResponse {
        let resp = self
            .http
            .post(format!("{}/event", self.base_url))
            .json(&body)
            .send()
            .await
            .expect("POST /event failed");

        assert_eq!(
            resp.status(),
            StatusCode::CREATED,
            "POST /event returned {}",
            resp.status()
        );

        resp.json().await.expect("invalid event response JSON")
    }

    pub async fn get_timeline(&self, session_id: &str) -> TimelineResponse {
        let resp = self
            .http
            .get(format!("{}/timeline", self.base_url))
            .query(&[("session_id", session_id)])
            .send()
            .await
            .expect("GET /timeline failed");

        assert_eq!(
            resp.status(),
            StatusCode::OK,
            "GET /timeline returned {}",
            resp.status()
        );

        resp.json().await.expect("invalid timeline response JSON")
    }

    pub async fn get_session(&self, session_id: &str) -> SessionResponse {
        let resp = self
            .http
            .get(format!("{}/session/{session_id}", self.base_url))
            .send()
            .await
            .expect("GET /session failed");

        assert_eq!(
            resp.status(),
            StatusCode::OK,
            "GET /session returned {}",
            resp.status()
        );

        resp.json().await.expect("invalid session response JSON")
    }

    pub async fn get_project_sessions(&self, project_id: &str) -> ProjectSessionsResponse {
        let resp = self
            .http
            .get(format!("{}/project/{project_id}/sessions", self.base_url))
            .send()
            .await
            .expect("GET /project/sessions failed");

        assert_eq!(
            resp.status(),
            StatusCode::OK,
            "GET /project/sessions returned {}",
            resp.status()
        );

        resp.json()
            .await
            .expect("invalid project sessions response JSON")
    }

    pub async fn post_explain(&self, session_id: &str) -> ExplainSnapshot {
        let resp = self
            .http
            .post(format!("{}/explain", self.base_url))
            .json(&json!({ "session_id": session_id }))
            .send()
            .await
            .expect("POST /explain failed");

        assert_eq!(
            resp.status(),
            StatusCode::OK,
            "POST /explain returned {}",
            resp.status()
        );

        let body: ExplainSnapshot = resp.json().await.expect("invalid explain response JSON");
        assert_explain_structure(&body);
        body
    }

    pub async fn get_integrity(&self) -> IntegrityResponse {
        let resp = self
            .http
            .get(format!("{}/debug/integrity", self.base_url))
            .send()
            .await
            .expect("GET /debug/integrity failed");

        assert_eq!(
            resp.status(),
            StatusCode::OK,
            "GET /debug/integrity returned {}",
            resp.status()
        );

        resp.json().await.expect("invalid integrity response JSON")
    }
}

pub fn event(session_id: &str, timestamp: i64, event_type: &str, payload: Value) -> Value {
    json!({
        "project_id": GOLDEN_PROJECT,
        "session_id": session_id,
        "timestamp": timestamp,
        "event_type": event_type,
        "payload": payload,
    })
}

pub fn assert_timeline_contains(
    timeline: &TimelineResponse,
    timestamps: &[i64],
    expected_types: &[&str],
) {
    assert_eq!(timeline.events.len(), expected_types.len());

    let types: Vec<&str> = timeline
        .events
        .iter()
        .map(|e| e.event_type.as_str())
        .collect();
    assert_eq!(types, expected_types);

    let ts: Vec<i64> = timeline.events.iter().map(|e| e.timestamp).collect();
    assert_eq!(ts, timestamps);
}

pub fn assert_explain_structure(explain: &ExplainSnapshot) {
    assert!(!explain.summary.is_empty(), "summary must not be empty");
    assert!(
        !explain.root_cause.is_empty(),
        "root_cause must not be empty"
    );
    assert!(
        !explain.problem_statement.is_empty(),
        "problem_statement must not be empty"
    );
    assert!(
        !explain.resolution_flow.is_empty(),
        "resolution_flow must not be empty"
    );
    assert!(
        !explain.decision_chain.is_empty(),
        "decision_chain must not be empty"
    );
}

pub fn snapshot_explain(name: &str, explain: &ExplainSnapshot) {
    insta::with_settings!({
        snapshot_path => "snapshots",
    }, {
        insta::assert_json_snapshot!(name, explain);
    });
}

#[derive(Debug, Deserialize)]
pub struct EventResponse {
    pub event: EventBody,
}

#[derive(Debug, Deserialize)]
pub struct EventBody {
    pub id: String,
    pub timestamp: i64,
    pub project_id: String,
    pub session_id: String,
    pub event_type: String,
    pub payload: Value,
}

#[derive(Debug, Deserialize)]
pub struct TimelineResponse {
    pub session_id: String,
    pub events: Vec<EventBody>,
    pub summary: Value,
}

#[derive(Debug, Deserialize)]
pub struct SessionResponse {
    pub session: SessionBody,
}

#[derive(Debug, Deserialize)]
pub struct SessionBody {
    pub id: String,
    pub project_id: String,
    pub started_at: i64,
    pub ended_at: Option<i64>,
    pub event_count: i64,
}

#[derive(Debug, Deserialize)]
pub struct ProjectSessionsResponse {
    pub project_id: String,
    pub sessions: Vec<SessionBody>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplainSnapshot {
    pub summary: String,
    pub root_cause: String,
    pub decision_chain: Vec<String>,
    pub problem_statement: String,
    pub resolution_flow: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IntegrityResponse {
    pub status: String,
    pub jsonl_line_count: u64,
    pub sqlite_indexed_count: u64,
    pub orphan_jsonl_count: u64,
    pub missing_offset_count: u64,
    pub broken_session_ids: Vec<String>,
    pub ordering_issue_count: u64,
    pub repaired_count: u64,
}

pub struct ScenarioBuilder {
    client: Client,
    session_id: String,
    events: Vec<Value>,
    next_timestamp: i64,
}

impl ScenarioBuilder {
    pub fn new(client: &Client, session_id: &str, start_timestamp: i64) -> Self {
        Self {
            client: client.clone(),
            session_id: session_id.to_string(),
            events: Vec::new(),
            next_timestamp: start_timestamp,
        }
    }

    pub fn push(mut self, event_type: &str, payload: Value) -> Self {
        self.events.push(event(
            &self.session_id,
            self.next_timestamp,
            event_type,
            payload,
        ));
        self.next_timestamp += 1;
        self
    }

    pub async fn send_all(self) -> TimelineResponse {
        for body in &self.events {
            self.client.post_event(body.clone()).await;
        }
        self.client.get_timeline(&self.session_id).await
    }
}
