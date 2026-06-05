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
use tower_http::services::ServeDir;
use tower_sessions::{MemoryStore, Session, SessionManagerLayer};

#[derive(Clone, Debug)]
struct AuthConfig {
    client_id: String,
    client_secret: String,
    issuer_url: String,
    redirect_uri: String,
    logout_uri: String,
    post_logout_redirect_uri: String,
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
        }
    }
}

#[derive(Clone)]
struct AppState {
    auth_config: AuthConfig,
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

    let state = AppState { auth_config };

    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store).with_secure(true);

    let app = Router::new()
        .route("/auth/login", get(login))
        .route("/auth/callback/authentik", get(auth_callback))
        .route("/auth/logout", get(logout))
        .route("/api/me", get(me))
        .fallback_service(ServeDir::new("dist"))
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
