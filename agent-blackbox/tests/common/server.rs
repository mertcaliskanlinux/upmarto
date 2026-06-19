use std::time::Duration;

use tempfile::TempDir;
use tokio::task::JoinHandle;
use tokio::time::sleep;
use upmarto::config::Config;
use upmarto::spawn_server;

use super::Client;

pub struct TestContext {
    pub client: Client,
    test_id: String,
    _guard: ServerGuard,
}

struct ServerGuard {
    _data_dir: TempDir,
    handle: JoinHandle<()>,
}

impl Drop for ServerGuard {
    fn drop(&mut self) {
        self.handle.abort();
    }
}

impl TestContext {
    pub async fn new(test_id: &str) -> Self {
        let data_dir = tempfile::tempdir().expect("failed to create isolated test data directory");
        let config = Config::for_isolated_test(data_dir.path(), 0);

        let (addr, handle) = spawn_server(config)
            .await
            .expect("failed to spawn isolated test server");

        let base_url = format!("http://{addr}");
        wait_for_server(&base_url).await;

        Self {
            client: Client::new(base_url),
            test_id: test_id.to_string(),
            _guard: ServerGuard {
                _data_dir: data_dir,
                handle,
            },
        }
    }

    /// Deterministic, test-scoped session id (e.g. `test_bug_fix_loop_session`).
    pub fn session(&self, role: &str) -> String {
        format!("{}_{}", self.test_id, role)
    }
}

async fn wait_for_server(base_url: &str) {
    let client = reqwest::Client::new();
    let url = format!("{base_url}/config");

    for _ in 0..100 {
        if let Ok(resp) = client.get(&url).send().await {
            if resp.status().is_success() {
                return;
            }
        }
        sleep(Duration::from_millis(25)).await;
    }

    panic!("test server failed to become ready at {base_url}");
}
