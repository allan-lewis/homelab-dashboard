use gloo_net::http::Request;

use crate::frontend::components::summary_panel::{SummaryPanelData, SummaryPanelItem};
use crate::frontend::models::{LiveCheckStatus, LiveState, SloCheckStatus};

pub async fn fetch_slo_status() -> Vec<SloCheckStatus> {
    match Request::get("/api/public/slo-status").send().await {
        Ok(response) => response
            .json::<Vec<SloCheckStatus>>()
            .await
            .unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}

pub async fn fetch_live_status() -> Vec<LiveCheckStatus> {
    match Request::get("/api/public/live-status").send().await {
        Ok(response) => response
            .json::<Vec<LiveCheckStatus>>()
            .await
            .unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}

pub fn live_pill_class(state: &LiveState) -> &'static str {
    match state {
        LiveState::Healthy => "status-pill up",
        LiveState::Unstable => "status-pill warning",
        LiveState::Down => "status-pill down",
    }
}

pub fn live_pill_label(state: &LiveState) -> &'static str {
    match state {
        LiveState::Healthy => "Healthy",
        LiveState::Unstable => "Unstable",
        LiveState::Down => "Down",
    }
}

/// Three-nines (99.9%) is the SLO target, so the headline status is a
/// strict pass/fail against it rather than a three-way RAG band.
const SLO_TARGET: f64 = 0.999;

pub fn slo_pill_class(pct: f64) -> &'static str {
    if pct >= SLO_TARGET {
        "status-pill up"
    } else {
        "status-pill down"
    }
}

pub fn slo_status_lines(checks: &[SloCheckStatus]) -> Vec<String> {
    let unmet_count = checks
        .iter()
        .filter(|check| check.pct_28d < SLO_TARGET)
        .count();

    if unmet_count == 0 {
        Vec::new()
    } else if unmet_count == 1 {
        vec!["1 SLO is below 99.9%.".to_string()]
    } else {
        vec![format!("{unmet_count} SLOs are below 99.9%.")]
    }
}

pub fn slo_info_lines(checks: &[SloCheckStatus]) -> Vec<String> {
    vec![format!("{} SLOs are being tracked.", checks.len())]
}

pub fn slo_summary_panel(checks: &[SloCheckStatus]) -> SummaryPanelData {
    let unmet_count = checks
        .iter()
        .filter(|check| check.pct_28d < SLO_TARGET)
        .count();

    let met_count = checks.len() - unmet_count;

    let mut items = Vec::new();

    if unmet_count > 0 {
        items.push(SummaryPanelItem {
            label: "SLOs not meeting target",
            count: unmet_count,
            pill_class: "status-pill down",
        });
    }

    if met_count > 0 {
        items.push(SummaryPanelItem {
            label: "SLOs meeting target",
            count: met_count,
            pill_class: "status-pill up",
        });
    }

    SummaryPanelData {
        title: "SLOs",
        empty_message: "No SLO checks configured yet.",
        items,
    }
}

pub fn slo_day_box_class(pct: Option<f64>) -> &'static str {
    match pct {
        None => "slo-day-box unknown",
        Some(pct) if pct >= 1.0 => "slo-day-box up",
        Some(pct) if pct >= 0.99 => "slo-day-box warning",
        Some(_) => "slo-day-box down",
    }
}
