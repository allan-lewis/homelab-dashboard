use gloo_net::http::Request;

use crate::frontend::components::summary_panel::{SummaryPanelData, SummaryPanelItem};
use crate::frontend::models::PrometheusTarget;

pub fn up_summary_panel(targets: &[PrometheusTarget]) -> SummaryPanelData {
    let down_count = targets
        .iter()
        .filter(|target| target_is_down(target))
        .count();

    let up_count = targets.len() - down_count;

    let mut items = Vec::new();

    if down_count > 0 {
        items.push(SummaryPanelItem {
            label: "Targets down",
            count: down_count,
            pill_class: "status-pill down",
        });
    }

    if up_count > 0 {
        items.push(SummaryPanelItem {
            label: "Targets up",
            count: up_count,
            pill_class: "status-pill up",
        });
    }

    SummaryPanelData {
        title: "Prometheus Targets",
        empty_message: "No targets found.",
        items,
    }
}

pub async fn fetch_targets() -> Vec<PrometheusTarget> {
    let mut targets = match Request::get("/api/up").send().await {
        Ok(response) => response
            .json::<Vec<PrometheusTarget>>()
            .await
            .unwrap_or_default(),
        Err(_) => Vec::new(),
    };

    targets.sort_by(|a, b| {
        a.instance
            .cmp(&b.instance)
            .then_with(|| a.job.cmp(&b.job))
    });

    targets
}

pub fn target_is_down(target: &PrometheusTarget) -> bool {
    !target.up
}
