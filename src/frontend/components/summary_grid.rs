use leptos::prelude::*;

use crate::frontend::components::summary_panel::{SummaryPanel, SummaryPanelState};

#[component]
pub fn SummaryGrid(panels: Vec<SummaryPanelState>) -> impl IntoView {
    view! {
        <div class="summary-grid">
            {panels
                .into_iter()
                .map(|panel| {
                    view! {
                        <SummaryPanel
                            loading=panel.loading
                            data=panel.data
                        />
                    }
                })
                .collect_view()}
        </div>
    }
}
