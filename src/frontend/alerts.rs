use crate::frontend::components::summary_panel::{SummaryPanelData, SummaryPanelItem};
use crate::frontend::models::FiringAlert;

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
