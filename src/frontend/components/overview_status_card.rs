use leptos::prelude::*;

#[component]
pub fn OverviewStatusCard() -> impl IntoView {
    view! {
        <section class="overview-card overview-status-card">
            <h3>"Status"</h3>
            <p class="overview-card-primary">
                "All monitored systems look healthy."
            </p>
            <p class="overview-card-secondary">
                "This is the left-hand overview status area."
            </p>
        </section>
    }
}
