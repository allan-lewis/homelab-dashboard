use leptos::prelude::*;

#[component]
pub fn UptimePage() -> impl IntoView {
    view! {
        <section class="page-content">
            <h2>"Uptime"</h2>
            <p>"This page will show uptime, reboot history, and hosts that may need attention after upgrades."</p>
        </section>
    }
}

