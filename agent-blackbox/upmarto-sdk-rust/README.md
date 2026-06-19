# upmarto-sdk

[![CI](https://github.com/mertcaliskanlinux/upmarto/actions/workflows/ci.yml/badge.svg)](https://github.com/mertcaliskanlinux/upmarto/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://github.com/mertcaliskanlinux/upmarto/blob/main/agent-blackbox/LICENSE)
[![crates.io](https://img.shields.io/crates/v/upmarto-sdk.svg)](https://crates.io/crates/upmarto-sdk)

Rust SDK for the Upmarto v1 API — event capture, offline queue, retry, and bootstrap.

## Installation

```bash
cargo add upmarto-sdk
```

## Usage

```rust
use upmarto_sdk::{EventType, TrackEvent, Upmarto};
use serde_json::json;

#[tokio::main]
async fn main() -> upmarto_sdk::Result<()> {
    let client = Upmarto::from_workspace(".").await?;
    client.session("my-session").await;
    client.track(TrackEvent {
        event_type: EventType::FileModified,
        payload: json!({ "path": "src/main.rs" }),
        timestamp: None,
    }).await?;
    client.flush().await?;
    Ok(())
}
```

## Features

- Frozen v1 endpoints: `POST /event`, `GET /timeline`, `POST /explain`
- Local queue persistence (`.upmarto/queue.jsonl`)
- Exponential backoff retry
- Workspace bootstrap and backend auto-discovery

## Documentation

- [SDK guide](https://github.com/mertcaliskanlinux/upmarto/blob/main/agent-blackbox/docs/SDK.md)
- [API contract](https://github.com/mertcaliskanlinux/upmarto/blob/main/agent-blackbox/docs/API_CONTRACT.md)
- [docs.rs](https://docs.rs/upmarto-sdk) (after publish)

## License

MIT — see [LICENSE](../LICENSE).
