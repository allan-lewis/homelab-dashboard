use leptos::prelude::*;

use crate::frontend::components::overview_dashboard::OverviewDashboard;

#[component]
pub fn OverviewPage() -> impl IntoView {
    view! {
        <section class="page-content">
            <h2>"Overview"</h2>

            <OverviewDashboard />
        </section>
    }
}
