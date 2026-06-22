use gloo_net::http::Request;

use crate::frontend::components::summary_panel::{SummaryPanelData, SummaryPanelItem};
use crate::frontend::models::FiringAlert;

pub fn alert_info_lines(alerts: &[FiringAlert]) -> Vec<String> {
    let info_count = alerts
        .iter()
        .filter(|alert| alert.severity == "info")
        .count();

    vec![format!("{info_count} info alerts active.")]
}

pub async fn fetch_alerts() -> Vec<FiringAlert> {
    let mut alerts = match Request::get("/api/alerts").send().await {
        Ok(response) => response.json::<Vec<FiringAlert>>().await.unwrap_or_default(),
        Err(_) => Vec::new(),
    };

    sort_alerts(&mut alerts);

    alerts
}

fn severity_rank(severity: &str) -> u8 {
    match severity {
        "critical" => 0,
        "warning" => 1,
        "info" => 2,
        _ => 3,
    }
}

pub fn sort_alerts(alerts: &mut [FiringAlert]) {
    alerts.sort_by(|a, b| {
        severity_rank(&a.severity)
            .cmp(&severity_rank(&b.severity))
            .then_with(|| a.alertname.cmp(&b.alertname))
            .then_with(|| a.rulegroup.cmp(&b.rulegroup))
            .then_with(|| a.instance.cmp(&b.instance))
    });
}

pub fn alert_summary_panel(alerts: &[FiringAlert]) -> SummaryPanelData {
    let critical_count = alerts
        .iter()
        .filter(|alert| alert.severity == "critical")
        .count();

    let warning_count = alerts
        .iter()
        .filter(|alert| alert.severity == "warning")
        .count();

    let info_count = alerts
        .iter()
        .filter(|alert| alert.severity == "info")
        .count();

    let mut items = Vec::new();

    if critical_count > 0 {
        items.push(SummaryPanelItem {
            label: "Critical alerts firing",
            count: critical_count,
            pill_class: "status-pill down",
        });
    }

    if warning_count > 0 {
        items.push(SummaryPanelItem {
            label: "Warning alerts firing",
            count: warning_count,
            pill_class: "status-pill warning",
        });
    }

    if info_count > 0 {
        items.push(SummaryPanelItem {
            label: "Info alerts firing",
            count: info_count,
            pill_class: "status-pill info",
        });
    }

    SummaryPanelData {
        title: "Alerts",
        empty_message: "No alerts firing.",
        items,
    }
}
