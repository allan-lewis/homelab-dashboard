use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect},
    routing::get,
    Json, Router,
};
use openidconnect::{
    core::{CoreAuthenticationFlow, CoreClient, CoreProviderMetadata},
    AuthorizationCode, ClientId, ClientSecret, CsrfToken, IssuerUrl, Nonce,
    RedirectUrl, Scope, TokenResponse,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use tokio::{
    sync::RwLock,
    time::{sleep, Duration},
};
use tower_http::services::{ServeDir, ServeFile};
use tower_sessions::{MemoryStore, Session, SessionManagerLayer};

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "lowercase")]
enum HostState {
    Up,
    Down,
    // Unknown,
}

#[derive(Clone, Debug, Serialize)]
struct HostStatus {
    hostname: String,
    persona: String,
    ip_address: String,
    status: HostState,
    timestamp: f64,
}

#[derive(Clone, Debug)]
struct AuthConfig {
    client_id: String,
    client_secret: String,
    issuer_url: String,
    redirect_uri: String,
    logout_uri: String,
    post_logout_redirect_uri: String,
    prometheus_url: String,
}

impl AuthConfig {
    fn from_env() -> Self {
        Self {
            client_id: std::env::var("HOMELAB_DASHBOARD_AUTHENTIK_CLIENT_ID")
                .expect("missing HOMELAB_DASHBOARD_AUTHENTIK_CLIENT_ID"),
            client_secret: std::env::var("HOMELAB_DASHBOARD_AUTHENTIK_CLIENT_SECRET")
                .expect("missing HOMELAB_DASHBOARD_AUTHENTIK_CLIENT_SECRET"),
            issuer_url: std::env::var("HOMELAB_DASHBOARD_AUTHENTIK_ISSUER_URL")
                .expect("missing HOMELAB_DASHBOARD_AUTHENTIK_ISSUER_URL"),
            redirect_uri: std::env::var("HOMELAB_DASHBOARD_AUTHENTIK_REDIRECT_URI")
                .expect("missing HOMELAB_DASHBOARD_AUTHENTIK_REDIRECT_URI"),
            logout_uri: std::env::var("HOMELAB_DASHBOARD_AUTHENTIK_LOGOUT_URI")
                .expect("missing HOMELAB_DASHBOARD_AUTHENTIK_LOGOUT_URI"),
            post_logout_redirect_uri: std::env::var("HOMELAB_DASHBOARD_POST_LOGOUT_REDIRECT_URI")
                .expect("missing HOMELAB_DASHBOARD_POST_LOGOUT_REDIRECT_URI"),
            prometheus_url: std::env::var("HOMELAB_DASHBOARD_PROMETHEUS_URL")
                .expect("missing HOMELAB_DASHBOARD_PROMETHEUS_URL"),
        }
    }
}

#[derive(Clone)]
struct AppState {
    auth_config: AuthConfig,
    host_up_cache: Arc<RwLock<Vec<HostUpStatus>>>,
    gatus_host_cache: Arc<RwLock<Vec<GatusHostStatus>>>,
}

#[derive(Debug, Deserialize)]
struct PrometheusQueryResponse {
    data: PrometheusData,
}

#[derive(Debug, Deserialize)]
struct PrometheusData {
    result: Vec<PrometheusResult>,
}

#[derive(Debug, Deserialize)]
struct PrometheusResult {
    metric: HashMap<String, String>,
    value: (f64, String),
}

#[derive(Clone, Debug, Serialize)]
struct GatusHostStatus {
    instance: String,
    name: String,
    target: String,
    timestamp: f64,
    up: bool,
}

#[derive(Clone, Debug, Serialize)]
struct HostUpStatus {
    instance: String,
    job: String,
    target: String,
    timestamp: f64,
    up: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct User {
    name: String,
    email: String,
}

#[derive(Debug, Deserialize)]
struct AuthCallbackParams {
    code: String,
    state: String,
}

fn redacted(value: &str) -> &'static str {
    if value.is_empty() {
        "<empty>"
    } else {
        "<redacted>"
    }
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
        IssuerUrl::new(state.auth_config.issuer_url.clone()).expect("invalid issuer URL"),
        &http_client,
    )
    .await
    .expect("failed to discover OIDC provider metadata");

    let oidc_client = CoreClient::from_provider_metadata(
        provider_metadata,
        ClientId::new(state.auth_config.client_id.clone()),
        Some(ClientSecret::new(state.auth_config.client_secret.clone())),
    )
    .set_redirect_uri(
        RedirectUrl::new(state.auth_config.redirect_uri.clone()).expect("invalid redirect URI"),
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
        IssuerUrl::new(state.auth_config.issuer_url.clone()).map_err(internal_error)?,
        &http_client,
    )
    .await
    .map_err(internal_error)?;

    let oidc_client = CoreClient::from_provider_metadata(
        provider_metadata,
        ClientId::new(state.auth_config.client_id.clone()),
        Some(ClientSecret::new(state.auth_config.client_secret.clone())),
    )
    .set_redirect_uri(
        RedirectUrl::new(state.auth_config.redirect_uri.clone()).map_err(internal_error)?,
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

// fn derive_hosts_from_up(statuses: &[HostUpStatus]) -> Vec<HostStatus> {
//     let mut grouped: HashMap<String, Vec<&HostUpStatus>> = HashMap::new();
//
//     for status in statuses {
//         grouped
//             .entry(status.instance.clone())
//             .or_default()
//             .push(status);
//     }
//
//     let mut hosts = grouped
//         .into_iter()
//         .map(|(hostname, entries)| {
//             let all_up = entries.iter().all(|entry| entry.up);
//             let all_down = entries.iter().all(|entry| !entry.up);
//
//             let status = if all_down {
//                 0
//             } else if all_up {
//                 2
//             } else {
//                 1
//             };
//
//             let timestamp = entries
//                 .iter()
//                 .map(|entry| entry.timestamp)
//                 .fold(0.0, f64::max);
//
//             let ip_address = entries
//                 .iter()
//                 .find_map(|entry| entry.target.split_once(':').map(|(ip, _)| ip.to_string()))
//                 .unwrap_or_default();
//
//             HostStatus {
//                 hostname,
//                 ip_address,
//                 status,
//                 timestamp,
//             }
//         })
//         .collect::<Vec<_>>();
//
//     hosts.sort_by(|a, b| a.hostname.cmp(&b.hostname));
//
//     hosts
// }

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

async fn fetch_gatus_hosts(
    prometheus_url: &str,
    client: &reqwest::Client,
) -> Result<Vec<GatusHostStatus>, String> {
    let response = client
        .get(format!("{}/api/v1/query", prometheus_url))
        .query(&[("query", r#"gatus_results_endpoint_success{group="Hosts"}"#)])
        .send()
        .await
        .map_err(|err| format!("failed to query Prometheus: {err}"))?;

    let prometheus_response = response
        .json::<PrometheusQueryResponse>()
        .await
        .map_err(|err| format!("failed to parse Prometheus response: {err}"))?;

    let mut statuses = prometheus_response
        .data
        .result
        .into_iter()
        .map(|result| {
            let name = result
                .metric
                .get("name")
                .cloned()
                .unwrap_or_else(|| "unknown".to_string());

            let instance = hostname_from_name(&name);

            let target = result
                .metric
                .get("target")
                .cloned()
                .unwrap_or_default();

            GatusHostStatus {
                instance,
                name,
                target,
                timestamp: result.value.0,
                up: result.value.1 == "1",
            }
        })
        .collect::<Vec<_>>();

    statuses.sort_by(|a, b| a.instance.cmp(&b.instance));

    Ok(statuses)
}

async fn fetch_prometheus_up(
    prometheus_url: &str,
    client: &reqwest::Client,
) -> Result<Vec<HostUpStatus>, String> {
    let response = client
        .get(format!("{}/api/v1/query", prometheus_url))
        .query(&[("query", "up")])
        .send()
        .await
        .map_err(|err| format!("failed to query Prometheus: {err}"))?;

    let prometheus_response = response
        .json::<PrometheusQueryResponse>()
        .await
        .map_err(|err| format!("failed to parse Prometheus response: {err}"))?;

    let mut statuses = prometheus_response
        .data
        .result
        .into_iter()
        .map(|result| {
            let instance = result
                .metric
                .get("instance")
                .cloned()
                .unwrap_or_else(|| "unknown".to_string());

            let job = result
                .metric
                .get("job")
                .cloned()
                .unwrap_or_else(|| "unknown".to_string());

            let target = result
                .metric
                .get("target")
                .cloned()
                .unwrap_or_default();

            HostUpStatus {
                instance,
                job,
                target,
                timestamp: result.value.0,
                up: result.value.1 == "1",
            }
        })
        .collect::<Vec<_>>();

    statuses.sort_by(|a, b| {
        a.instance
            .cmp(&b.instance)
            .then_with(|| a.job.cmp(&b.job))
            .then_with(|| a.target.cmp(&b.target))
    });

    Ok(statuses)
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

fn hostname_from_name(name: &str) -> String {
    name.split_whitespace()
        .next()
        .unwrap_or("unknown")
        .to_lowercase()
}

fn persona_from_name(name: &str) -> String {
    let Some(start) = name.find('(') else {
        return String::new();
    };

    let Some(end) = name[start + 1..].find(')') else {
        return String::new();
    };

    name[start + 1..start + 1 + end].to_string()
}

fn ip_address_for_host(hostname: &str, up_statuses: &[HostUpStatus]) -> String {
    up_statuses
        .iter()
        .find(|status| status.instance == hostname)
        .and_then(|status| status.target.split_once(':').map(|(ip, _)| ip.to_string()))
        .unwrap_or_default()
}

#[tokio::main]
async fn main() {
    let auth_config = AuthConfig::from_env();

    println!("Loaded Authentik config:");
    println!("  issuer_url: {}", auth_config.issuer_url);
    println!("  redirect_uri: {}", auth_config.redirect_uri);
    println!("  logout_uri: {}", auth_config.logout_uri);
    println!(
        "  post_logout_redirect_uri: {}",
        auth_config.post_logout_redirect_uri
    );
    println!("  client_id: {}", redacted(&auth_config.client_id));
    println!("  client_secret: {}", redacted(&auth_config.client_secret));
    println!("  prometheus_url: {}", auth_config.prometheus_url);

    let host_up_cache = Arc::new(RwLock::new(Vec::<HostUpStatus>::new()));
    let gatus_host_cache = Arc::new(RwLock::new(Vec::<GatusHostStatus>::new()));

    start_prometheus_up_polling(
        auth_config.prometheus_url.clone(),
        host_up_cache.clone(),
    );

    start_gatus_host_polling(
        auth_config.prometheus_url.clone(),
        gatus_host_cache.clone(),
    );

    let state = AppState {
        auth_config,
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
