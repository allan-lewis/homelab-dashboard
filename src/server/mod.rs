mod auth;
mod cache;
mod config;
mod handlers;
mod metrics;
mod models;
mod util;

use axum::{Router, routing::get};
use cache::start_polling;
use config::AppConfig;
use metrics::{fetch_certificate_expiries, fetch_firing_alerts, fetch_gatus_hosts, fetch_prometheus_up, fetch_homelab_tasks};
use models::{CertificateExpiry, FiringAlert, GatusHostStatus, HomelabTask, HostUpStatus};
use std::sync::Arc;
use std::time::Duration as StdDuration;
use tokio::sync::RwLock;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;
use tower_sessions::{MemoryStore, SessionManagerLayer};
use tracing::{Level, info};
use tracing_subscriber::EnvFilter;

#[derive(Clone)]
struct AppState {
    config: AppConfig,
    certificate_expiry_cache: Arc<RwLock<Vec<CertificateExpiry>>>,
    firing_alerts_cache: Arc<RwLock<Vec<FiringAlert>>>,
    host_up_cache: Arc<RwLock<Vec<HostUpStatus>>>,
    gatus_host_cache: Arc<RwLock<Vec<GatusHostStatus>>>,
    homelab_tasks_cache: Arc<RwLock<Vec<HomelabTask>>>,
}

fn sanitized_uri(uri: &axum::http::Uri) -> String {
    uri.path().to_string()
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
        .route("/api/alerts", get(handlers::alerts))
        .route("/api/certificates", get(handlers::certificates))
        .route("/api/tasks", get(handlers::tasks))
        .fallback_service(ServeDir::new("dist").fallback(ServeFile::new("dist/index.html")))
        .with_state(state)
        .layer(session_layer)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &axum::http::Request<_>| {
                    tracing::span!(
                        Level::INFO,
                        "http_request",
                        method = %request.method(),
                        uri = %sanitized_uri(request.uri()),
                    )
                })
                .on_response(
                    |response: &axum::http::Response<_>,
                     latency: StdDuration,
                     _span: &tracing::Span| {
                        info!(
                            status = response.status().as_u16(),
                            latency_ms = latency.as_millis(),
                            "request completed"
                        );
                    },
                ),
        )
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("server=info,tower_http=info"));

    tracing_subscriber::fmt().with_env_filter(filter).init();
}

pub async fn run() {
    init_tracing();

    let config = AppConfig::from_env();

    info!(
        issuer_url = %config.auth.issuer_url,
        redirect_uri = %config.auth.redirect_uri,
        logout_uri = %config.auth.logout_uri,
        post_logout_redirect_uri = %config.auth.post_logout_redirect_uri,
        client_id = config::redacted(&config.auth.client_id),
        client_secret = config::redacted(&config.auth.client_secret),
        prometheus_url = %config.prometheus.url,
        "loaded server config"
    );

    let certificate_expiry_cache = Arc::new(RwLock::new(Vec::<CertificateExpiry>::new()));
    let firing_alerts_cache = Arc::new(RwLock::new(Vec::<FiringAlert>::new()));
    let host_up_cache = Arc::new(RwLock::new(Vec::<HostUpStatus>::new()));
    let gatus_host_cache = Arc::new(RwLock::new(Vec::<GatusHostStatus>::new()));
    let homelab_tasks_cache = Arc::new(RwLock::new(Vec::<HomelabTask>::new()));

    start_polling(
        "certificate_expiry",
        config.prometheus.url.clone(),
        certificate_expiry_cache.clone(),
        fetch_certificate_expiries,
    );

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

    start_polling(
        "firing_alerts",
        config.prometheus.url.clone(),
        firing_alerts_cache.clone(),
        fetch_firing_alerts,
    );

    start_polling(
        "homelab_tasks",
        config.prometheus.url.clone(),
        homelab_tasks_cache.clone(),
        fetch_homelab_tasks,
    );

    let state = AppState {
        config,
        certificate_expiry_cache,
        firing_alerts_cache,
        host_up_cache,
        gatus_host_cache,
        homelab_tasks_cache
    };

    let app = router(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("failed to bind to port 3000");

    info!("listening on http://0.0.0.0:3000");

    axum::serve(listener, app).await.expect("server failed");
}
