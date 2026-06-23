use leptos::prelude::*;

#[component]
pub fn OverviewInfoCard(
    loading: bool,
    lines: Vec<String>,
    last_updated: ReadSignal<Option<String>>,
) -> impl IntoView {
    view! {
        <section class="overview-card overview-info-card">
            <h3>"Information"</h3>

            {if loading {
                view! {
                    <p>
                        "Loading overview data..."
                    </p>
                }.into_any()
            } else {
                view! {
                    <div>
                        {lines
                            .into_iter()
                            .map(|line| {
                                view! {
                                    <p>{line}</p>
                                }
                            })
                            .collect_view()}

                        {match last_updated.get() {
                            Some(updated) => view! {
                                <p class="overview-card-secondary">
                                    "Last updated " {updated} "."
                                </p>
                            }.into_any(),
                            None => view! {
                                <p class="overview-card-secondary">
                                    "Last updated time unavailable."
                                </p>
                            }.into_any(),
                        }}
                    </div>
                }.into_any()
            }}
        </section>
    }
}
