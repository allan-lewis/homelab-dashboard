use gloo_net::http::Request;

use crate::frontend::models::SloCheckStatus;

pub async fn fetch_slo_status() -> Vec<SloCheckStatus> {
    match Request::get("/api/public/slo-status").send().await {
        Ok(response) => response
            .json::<Vec<SloCheckStatus>>()
            .await
            .unwrap_or_default(),
        Err(_) => Vec::new(),
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

pub fn slo_day_box_class(pct: Option<f64>) -> &'static str {
    match pct {
        None => "slo-day-box unknown",
        Some(pct) if pct >= SLO_TARGET => "slo-day-box up",
        Some(_) => "slo-day-box down",
    }
}
