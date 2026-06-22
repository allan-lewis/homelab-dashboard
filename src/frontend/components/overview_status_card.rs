use leptos::prelude::*;

#[component]
pub fn OverviewStatusCard(
    loading: bool,
    lines: Vec<String>,
) -> impl IntoView {
    view! {
        <section class="overview-card overview-status-card">
            <h3>"Status"</h3>

            {if loading {
                view! {
                    <p class="overview-card-primary">
                        "Loading overview status..."
                    </p>
                }.into_any()
            } else if lines.is_empty() {
                view! {
                    <p class="overview-card-primary">
                        "All monitored systems look healthy."
                    </p>
                }.into_any()
            } else {
                view! {
                    <div class="overview-status-list">
                        {lines
                            .into_iter()
                            .map(|line| {
                                view! {
                                    <p>{line}</p>
                                }
                            })
                            .collect_view()}
                    </div>
                }.into_any()
            }}
        </section>
    }
}
