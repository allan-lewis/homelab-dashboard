mod auth;
mod cache;
mod config;
mod handlers;
mod metrics;
mod models;
mod util;

use cache::start_polling;
use config::{redacted, AppConfig};
use axum::{
    routing::get,
    Router,
};
use metrics::{fetch_gatus_hosts, fetch_prometheus_up};
use models::{
    GatusHostStatus, HostUpStatus,
};
use std::{sync::Arc};
use tokio::{
    sync::RwLock,
};
use tower_http::services::{ServeDir, ServeFile};
use tower_sessions::{MemoryStore, SessionManagerLayer};

#[derive(Clone)]
struct AppState {
    config: AppConfig,
    host_up_cache: Arc<RwLock<Vec<HostUpStatus>>>,
    gatus_host_cache: Arc<RwLock<Vec<GatusHostStatus>>>,
}

fn router(state: AppState) -> Router {
    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store).with_secure(true);

    Router::new()
        .route("/auth/login", get(auth::login))
        .route("/auth/callback/authentik", get(auth::auth_callback))
        .route("/auth/logout", get(auth::logout))
        .route("/api/me", get(handlers::me))
        .route("/api/prometheus/up", get(handlers::prometheus_up))
        .route("/api/hosts", get(handlers::hosts))
        .fallback_service(ServeDir::new("dist").fallback(ServeFile::new("dist/index.html")))
        .with_state(state)
        .layer(session_layer)
}

pub async fn run() {
    let config = AppConfig::from_env();

    println!("Loaded Authentik config:");
    println!("  issuer_url: {}", config.auth.issuer_url);
    println!("  redirect_uri: {}", config.auth.redirect_uri);
    println!("  logout_uri: {}", config.auth.logout_uri);
    println!(
        "  post_logout_redirect_uri: {}",
        config.auth.post_logout_redirect_uri
    );
    println!("  client_id: {}", redacted(&config.auth.client_id));
    println!("  client_secret: {}", redacted(&config.auth.client_secret));
    println!("  prometheus_url: {}", config.prometheus.url);

    let host_up_cache = Arc::new(RwLock::new(Vec::<HostUpStatus>::new()));
    let gatus_host_cache = Arc::new(RwLock::new(Vec::<GatusHostStatus>::new()));

    start_polling(
        "prometheus_up",
        config.prometheus.url.clone(),
        host_up_cache.clone(),
        fetch_prometheus_up,
    );

    start_polling(
        "gatus_hosts",
        config.prometheus.url.clone(),
        gatus_host_cache.clone(),
        fetch_gatus_hosts,
    );

    let state = AppState {
        config,
        host_up_cache,
        gatus_host_cache,
    };

    let app = router(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("failed to bind to port 3000");

    println!("Listening on http://0.0.0.0:3000");

    axum::serve(listener, app)
        .await
        .expect("server failed");
}
