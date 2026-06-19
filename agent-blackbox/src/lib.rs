pub mod api;
pub mod config;
pub mod core;
pub mod product;
pub mod reasoning;
pub mod replay;
pub mod services;
pub mod storage;
pub mod utils;

use std::net::SocketAddr;
use std::sync::{Arc, Once};

use tracing::info;

use api::routes::create_router;
use api::{AppState, RuntimeInfo};
use config::Settings;
use storage::repository::Repository;

static TRACING: Once = Once::new();

fn init_tracing() {
    TRACING.call_once(|| {
        tracing_subscriber::fmt()
            .with_env_filter(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| "upmarto=info,tower_http=info".into()),
            )
            .with_test_writer()
            .init();
    });
}

pub async fn run() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    run_with_config(Settings::from_env()).await
}

pub async fn run_with_config(
    settings: Settings,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    init_tracing();
    settings.ensure_storage_dirs()?;

    let repo = Arc::new(Repository::new(
        settings.events_log_path(),
        settings.sqlite_path(),
    )?);

    let bind_addr: SocketAddr = format!("{}:{}", settings.host, settings.port).parse()?;
    let listener = tokio::net::TcpListener::bind(bind_addr).await?;
    let bound_addr = listener.local_addr()?;

    let api_base_url = Settings::resolve_api_base_url(bound_addr, &settings.public_base_url);
    let runtime = RuntimeInfo {
        api_base_url: api_base_url.clone(),
        host: bound_addr.ip().to_string(),
        port: bound_addr.port(),
    };

    let state = AppState::new(repo, runtime);
    let app = create_router(state);

    info!("[Upmarto] listening on http://{bound_addr}");
    info!("API base URL: {api_base_url}");
    if settings.test_mode {
        info!(
            "TEST_MODE enabled — isolated storage at {}",
            settings.data_dir.display()
        );
    }
    info!("Events log: {}", settings.events_log_path().display());
    info!("Metadata DB: {}", settings.sqlite_path().display());

    axum::serve(listener, app).await?;

    Ok(())
}

/// Spawns a background server for integration tests. Aborts when the returned handle is dropped.
pub async fn spawn_server(
    settings: Settings,
) -> Result<(SocketAddr, tokio::task::JoinHandle<()>), Box<dyn std::error::Error + Send + Sync>> {
    init_tracing();
    settings.ensure_storage_dirs()?;

    let repo = Arc::new(Repository::new(
        settings.events_log_path(),
        settings.sqlite_path(),
    )?);

    let bind_addr: SocketAddr = format!("{}:{}", settings.host, settings.port).parse()?;
    let listener = tokio::net::TcpListener::bind(bind_addr).await?;
    let bound_addr = listener.local_addr()?;

    let api_base_url = Settings::resolve_api_base_url(bound_addr, &settings.public_base_url);
    let runtime = RuntimeInfo {
        api_base_url,
        host: bound_addr.ip().to_string(),
        port: bound_addr.port(),
    };

    let state = AppState::new(repo, runtime);
    let app = create_router(state);

    let handle = tokio::spawn(async move {
        if let Err(err) = axum::serve(listener, app).await {
            tracing::error!("server stopped: {err}");
        }
    });

    Ok((bound_addr, handle))
}
