use leptos::prelude::*;

#[component]
pub fn OverviewInfoCard() -> impl IntoView {
    view! {
        <section class="overview-card overview-info-card">
            <h3>"Information"</h3>
            <p class="overview-card-primary">
                "14 hosts reporting."
            </p>
            <p class="overview-card-secondary">
                "Last updated Monday, June 22 at 02:14:37 UTC."
            </p>
        </section>
    }
}
