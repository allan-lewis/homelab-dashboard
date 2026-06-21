use axum::{Json, extract::State};
use tower_sessions::Session;

use super::{
    AppState,
    models::{CertificateExpiry, FiringAlert, HostState, HostStatus, HostUpStatus, User},
    util::{ip_address_for_host, persona_from_name},
};

pub async fn me(session: Session) -> Json<Option<User>> {
    let user: Option<User> = session
        .get("user")
        .await
        .expect("failed to read user from session");

    Json(user)
}

pub async fn hosts(State(state): State<AppState>) -> Json<Vec<HostStatus>> {
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

pub async fn prometheus_up(State(state): State<AppState>) -> Json<Vec<HostUpStatus>> {
    let statuses = state.host_up_cache.read().await.clone();
    Json(statuses)
}

pub async fn alerts(State(state): State<AppState>) -> Json<Vec<FiringAlert>> {
    let alerts= state.firing_alerts_cache.read().await.clone();
    Json(alerts)
}

pub async fn certificates(State(state): State<AppState>) -> Json<Vec<CertificateExpiry>> {
    let certificates= state.certificate_expiry_cache.read().await.clone();
    Json(certificates)
}

