use crate::frontend::components::summary_panel::{SummaryPanelData, SummaryPanelItem};
use crate::frontend::models::{HostState, HostStatus};

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
