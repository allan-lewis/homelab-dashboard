mod auth;
mod config;
mod metrics;
mod models;
mod util;

use config::{redacted, AppConfig};
use axum::{
    extract::{State},
    response::{IntoResponse},
    routing::get,
    Json, Router,
};
use metrics::{fetch_gatus_hosts, fetch_prometheus_up};
use models::{
    GatusHostStatus, HostState, HostStatus, HostUpStatus, User,
};
use std::{sync::Arc};
use tokio::{
    sync::RwLock,
    time::{sleep, Duration},
};
use tower_http::services::{ServeDir, ServeFile};
use tower_sessions::{MemoryStore, Session, SessionManagerLayer};
use util::{ip_address_for_host, persona_from_name};

#[derive(Clone)]
struct AppState {
    config: AppConfig,
    host_up_cache: Arc<RwLock<Vec<HostUpStatus>>>,
    gatus_host_cache: Arc<RwLock<Vec<GatusHostStatus>>>,
}

async fn me(session: Session) -> impl IntoResponse {
    let user: Option<User> = session
        .get("user")
        .await
        .expect("failed to read user from session");

    Json(user)
}

async fn hosts(State(state): State<AppState>) -> Json<Vec<HostStatus>> {
    let gatus_statuses = state.gatus_host_cache.read().await;
    let up_statuses = state.host_up_cache.read().await;

    let mut hosts = gatus_statuses
        .iter()
        .map(|status| HostStatus {
            hostname: status.instance.clone(),
            persona: persona_from_name(&status.name),
            ip_address: ip_address_for_host(&status.instance, &up_statuses),
            status: if status.up {
                HostState::Up
            } else {
                HostState::Down
            },
            timestamp: status.timestamp,
        })
        .collect::<Vec<_>>();

    hosts.sort_by(|a, b| a.hostname.cmp(&b.hostname));

    Json(hosts)
}

async fn prometheus_up(State(state): State<AppState>) -> Json<Vec<HostUpStatus>> {
    let statuses = state.host_up_cache.read().await.clone();
    Json(statuses)
}

fn start_gatus_host_polling(
    prometheus_url: String,
    cache: Arc<RwLock<Vec<GatusHostStatus>>>,
) {
    tokio::spawn(async move {
        let client = reqwest::Client::new();

        loop {
            match fetch_gatus_hosts(&prometheus_url, &client).await {
                Ok(statuses) => {
                    println!("Updated Gatus host cache: {} entries", statuses.len());
                    *cache.write().await = statuses;
                }
                Err(err) => {
                    eprintln!("Failed to update Gatus host cache: {err}");
                }
            }

            sleep(Duration::from_secs(30)).await;
        }
    });
}

fn start_prometheus_up_polling(
    prometheus_url: String,
    cache: Arc<RwLock<Vec<HostUpStatus>>>,
) {
    tokio::spawn(async move {
        let client = reqwest::Client::new();

        loop {
            match fetch_prometheus_up(&prometheus_url, &client).await {
                Ok(statuses) => {
                    println!("Updated Prometheus up cache: {} entries", statuses.len());
                    *cache.write().await = statuses;
                }
                Err(err) => {
                    eprintln!("Failed to update Prometheus up cache: {err}");
                }
            }

            sleep(Duration::from_secs(30)).await;
        }
    });
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

    start_prometheus_up_polling(
        config.prometheus.url.clone(),
        host_up_cache.clone(),
    );

    start_gatus_host_polling(
        config.prometheus.url.clone(),
        gatus_host_cache.clone(),
    );

    let state = AppState {
        config,
        host_up_cache,
        gatus_host_cache,
    };

    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store).with_secure(true);

    let app = Router::new()
        .route("/auth/login", get(auth::login))
        .route("/auth/callback/authentik", get(auth::auth_callback))
        .route("/auth/logout", get(auth::logout))
        .route("/api/me", get(me))
        .route("/api/prometheus/up", get(prometheus_up))
        .route("/api/hosts", get(hosts))
        .fallback_service(
            ServeDir::new("dist").fallback(ServeFile::new("dist/index.html")),
        )
        .with_state(state)
        .layer(session_layer);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("failed to bind to port 3000");

    println!("Listening on http://0.0.0.0:3000");

    axum::serve(listener, app)
        .await
        .expect("server failed");
}
