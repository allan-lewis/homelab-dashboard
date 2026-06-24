use gloo_net::http::Request;

use crate::frontend::components::summary_panel::{SummaryPanelData, SummaryPanelItem};
use crate::frontend::models::NixosGeneration;

pub fn generation_is_current(generation: &NixosGeneration) -> bool {
    generation.booted_is_current
}

pub fn generation_summary_panel(generations: &[NixosGeneration]) -> SummaryPanelData {
    let not_current_count = generations
        .iter()
        .filter(|generation| !generation_is_current(generation))
        .count();

    let current_count = generations.len() - not_current_count;

    let mut items = Vec::new();

    if not_current_count > 0 {
        items.push(SummaryPanelItem {
            label: "Systems not latest booted",
            count: not_current_count,
            pill_class: "status-pill warning",
        });
    }

    if current_count > 0 {
        items.push(SummaryPanelItem {
            label: "Systems latest booted",
            count: current_count,
            pill_class: "status-pill up",
        });
    }

    SummaryPanelData {
        title: "NixOS Generations",
        empty_message: "No generation data found.",
        items,
    }
}

pub async fn fetch_generations() -> Vec<NixosGeneration> {
    let mut generations = match Request::get("/api/generations").send().await {
        Ok(response) => response
            .json::<Vec<NixosGeneration>>()
            .await
            .unwrap_or_default(),
        Err(_) => Vec::new(),
    };

    generations.sort_by(|a, b| a.instance.cmp(&b.instance));

    generations
}
