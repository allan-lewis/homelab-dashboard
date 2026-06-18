use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect},
};

use openidconnect::{
    core::{CoreAuthenticationFlow, CoreClient, CoreProviderMetadata},
    AuthorizationCode, ClientId, ClientSecret, CsrfToken, IssuerUrl, Nonce, RedirectUrl, Scope,
    TokenResponse,
};

use tower_sessions::Session;

use super::{
    models::{AuthCallbackParams, User},
    AppState,
};

fn internal_error(message: impl std::fmt::Display) -> (StatusCode, String) {
    eprintln!("Auth error: {message}");
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        "Authentication failed".to_string(),
    )
}

pub async fn login(State(state): State<AppState>, session: Session) -> impl IntoResponse {
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

pub async fn auth_callback(
    State(state): State<AppState>,
    session: Session,
    Query(params): Query<AuthCallbackParams>,
) -> Result<Redirect, (StatusCode, String)> {
    let expected_state: Option<String> = session.get("oauth_csrf").await.map_err(internal_error)?;

    let Some(expected_state) = expected_state else {
        return Err((StatusCode::BAD_REQUEST, "Missing OAuth state".to_string()));
    };

    if params.state != expected_state {
        return Err((StatusCode::BAD_REQUEST, "Invalid OAuth state".to_string()));
    }

    let nonce_value: Option<String> = session.get("oauth_nonce").await.map_err(internal_error)?;

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

    session.insert("user", user).await.map_err(internal_error)?;

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

pub async fn logout(session: Session) -> impl IntoResponse {
    session.delete().await.expect("failed to delete session");
    Redirect::to("/")
}
