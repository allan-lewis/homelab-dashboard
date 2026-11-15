use leptos::prelude::*;

use crate::frontend::components::overview_dashboard::OverviewDashboard;

#[component]
pub fn OverviewPage(set_last_updated: WriteSignal<Option<String>>) -> impl IntoView {
    view! {
        <section class="page-content">
            <h2>"Overview"</h2>

            <OverviewDashboard set_last_updated=set_last_updated />
        </section>
    }
}
