use gloo_net::http::Request;

use crate::frontend::components::summary_panel::{SummaryPanelData, SummaryPanelItem};
use crate::frontend::models::{HostState, HostStatus};

pub async fn fetch_hosts() -> Vec<HostStatus> {
    match Request::get("/api/hosts").send().await {
        Ok(response) => response.json::<Vec<HostStatus>>().await.unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}

pub fn host_info_lines(hosts: &[HostStatus]) -> Vec<String> {
    vec![format!("{} hosts reporting.", hosts.len())]
}

pub fn host_summary_panel(hosts: &[HostStatus]) -> SummaryPanelData {
    let down_count = hosts
        .iter()
        .filter(|host| matches!(host.status, HostState::Down))
        .count();

    let unknown_count = hosts
        .iter()
        .filter(|host| matches!(host.status, HostState::Unknown))
        .count();

    let up_count = hosts
        .iter()
        .filter(|host| matches!(host.status, HostState::Up))
        .count();

    let mut items = Vec::new();

    if down_count > 0 {
        items.push(SummaryPanelItem {
            label: "Hosts down",
            count: down_count,
            pill_class: "status-pill down",
        });
    }

    if unknown_count > 0 {
        items.push(SummaryPanelItem {
            label: "Hosts unknown",
            count: unknown_count,
            pill_class: "status-pill unknown",
        });
    }

    if up_count > 0 {
        items.push(SummaryPanelItem {
            label: "Hosts up",
            count: up_count,
            pill_class: "status-pill up",
        });
    }

    SummaryPanelData {
        title: "Hosts",
        empty_message: "No hosts found.",
        items,
    }
}
