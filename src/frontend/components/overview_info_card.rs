use leptos::prelude::*;

#[component]
pub fn OverviewInfoCard(
    loading: bool,
    lines: Vec<String>,
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
                    </div>
                }.into_any()
            }}
        </section>
    }
}
