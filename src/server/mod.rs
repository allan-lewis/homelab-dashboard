mod config;
mod metrics;
mod models;
mod util;

use config::{redacted, AppConfig};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect},
    routing::get,
    Json, Router,
};
use metrics::{fetch_gatus_hosts, fetch_prometheus_up};
use models::{
    AuthCallbackParams, GatusHostStatus, HostState, HostStatus, HostUpStatus,
    User,
};
use openidconnect::{
    core::{CoreAuthenticationFlow, CoreClient, CoreProviderMetadata},
    AuthorizationCode, ClientId, ClientSecret, CsrfToken, IssuerUrl, Nonce,
    RedirectUrl, Scope, TokenResponse,
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

fn internal_error(message: impl std::fmt::Display) -> (StatusCode, String) {
    eprintln!("Auth error: {message}");
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        "Authentication failed".to_string(),
    )
}

async fn login(State(state): State<AppState>, session: Session) -> impl IntoResponse {
    let http_client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("failed to build HTTP client");

    let provider_metadata = CoreProviderMetadata::discover_async(
        IssuerUrl::new(state.config.auth.issuer_url.clone()).expect("invalid issuer URL"),
        &http_client,
    )
    .await
    .expect("failed to discover OIDC provider metadata");

    let oidc_client = CoreClient::from_provider_metadata(
        provider_metadata,
        ClientId::new(state.config.auth.client_id.clone()),
        Some(ClientSecret::new(state.config.auth.client_secret.clone())),
    )
    .set_redirect_uri(
        RedirectUrl::new(state.config.auth.redirect_uri.clone()).expect("invalid redirect URI"),
    );

    let (auth_url, csrf_token, nonce) = oidc_client
        .authorize_url(
            CoreAuthenticationFlow::AuthorizationCode,
            CsrfToken::new_random,
            Nonce::new_random,
        )
        .add_scope(Scope::new("openid".to_string()))
        .add_scope(Scope::new("profile".to_string()))
        .add_scope(Scope::new("email".to_string()))
        .url();

    session
        .insert("oauth_csrf", csrf_token.secret())
        .await
        .expect("failed to save csrf token");

    session
        .insert("oauth_nonce", nonce.secret())
        .await
        .expect("failed to save nonce");

    Redirect::to(auth_url.as_str())
}

async fn auth_callback(
    State(state): State<AppState>,
    session: Session,
    Query(params): Query<AuthCallbackParams>,
) -> Result<Redirect, (StatusCode, String)> {
    let expected_state: Option<String> = session
        .get("oauth_csrf")
        .await
        .map_err(internal_error)?;

    let Some(expected_state) = expected_state else {
        return Err((StatusCode::BAD_REQUEST, "Missing OAuth state".to_string()));
    };

    if params.state != expected_state {
        return Err((StatusCode::BAD_REQUEST, "Invalid OAuth state".to_string()));
    }

    let nonce_value: Option<String> = session
        .get("oauth_nonce")
        .await
        .map_err(internal_error)?;

    let Some(nonce_value) = nonce_value else {
        return Err((StatusCode::BAD_REQUEST, "Missing OAuth nonce".to_string()));
    };

    let http_client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(internal_error)?;

    let provider_metadata = CoreProviderMetadata::discover_async(
        IssuerUrl::new(state.config.auth.issuer_url.clone()).map_err(internal_error)?,
        &http_client,
    )
    .await
    .map_err(internal_error)?;

    let oidc_client = CoreClient::from_provider_metadata(
        provider_metadata,
        ClientId::new(state.config.auth.client_id.clone()),
        Some(ClientSecret::new(state.config.auth.client_secret.clone())),
    )
    .set_redirect_uri(
        RedirectUrl::new(state.config.auth.redirect_uri.clone()).map_err(internal_error)?,
    );

    let token_response = oidc_client
        .exchange_code(AuthorizationCode::new(params.code))
        .map_err(internal_error)?
        .request_async(&http_client)
        .await
        .map_err(internal_error)?;

    let id_token = token_response
        .id_token()
        .ok_or_else(|| internal_error("provider did not return an ID token"))?;

    let id_token_verifier = oidc_client.id_token_verifier();
    let nonce = Nonce::new(nonce_value);

    let claims = id_token
        .claims(&id_token_verifier, &nonce)
        .map_err(internal_error)?;

    let name = claims
        .name()
        .and_then(|name| name.get(None))
        .map(|name| name.to_string())
        .unwrap_or_else(|| claims.subject().as_str().to_string());

    let email = claims
        .email()
        .map(|email| email.as_str().to_string())
        .unwrap_or_else(|| "".to_string());

    let user = User { name, email };

    session
        .insert("user", user)
        .await
        .map_err(internal_error)?;

    session
        .remove::<String>("oauth_csrf")
        .await
        .map_err(internal_error)?;

    session
        .remove::<String>("oauth_nonce")
        .await
        .map_err(internal_error)?;

    Ok(Redirect::to("/"))
}

async fn logout(session: Session) -> impl IntoResponse {
    session.delete().await.expect("failed to delete session");
    Redirect::to("/")
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
        .route("/auth/login", get(login))
        .route("/auth/callback/authentik", get(auth_callback))
        .route("/auth/logout", get(logout))
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
