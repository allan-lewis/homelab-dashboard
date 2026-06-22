use leptos::prelude::*;

use crate::frontend::components::summary_line::SummaryLine;

#[derive(Clone, Debug)]
pub struct SummaryPanelData {
    pub title: &'static str,
    pub empty_message: &'static str,
    pub items: Vec<SummaryPanelItem>,
}

#[derive(Clone, Debug)]
pub struct SummaryPanelItem {
    pub label: &'static str,
    pub count: usize,
    pub pill_class: &'static str,
}

#[derive(Clone, Debug)]
pub struct SummaryPanelState {
    pub loading: bool,
    pub data: SummaryPanelData,
}

#[component]
pub fn SummaryPanel(
    loading: bool,
    data: SummaryPanelData,
) -> impl IntoView {
    view! {
        <section class="summary-panel">
            <h3>{data.title}</h3>

            {if loading {
                view! { <p>"Loading..."</p> }.into_any()
            } else if data.items.is_empty() {
                view! { <p>{data.empty_message}</p> }.into_any()
            } else {
                view! {
                    <div class="summary-list">
                        {data.items
                            .into_iter()
                            .map(|item| {
                                view! {
                                    <SummaryLine
                                        label=item.label
                                        count=item.count
                                        pill_class=item.pill_class
                                    />
                                }
                            })
                            .collect_view()}
                    </div>
                }.into_any()
            }}
        </section>
    }
}
