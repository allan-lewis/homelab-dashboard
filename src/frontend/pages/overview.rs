use leptos::prelude::*;

#[component]
pub fn OverviewPage(name: String) -> impl IntoView {
    view! {
        <section class="page-content">
            <h2>"Overview"</h2>
            <p>"Welcome " {name}</p>
            <p>"This will show high-level fleet status: host count, unhealthy hosts, stale check-ins, and recent changes."</p>
        </section>
    }
}

