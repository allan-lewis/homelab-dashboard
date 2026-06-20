use leptos::prelude::*;

#[component]
pub fn GenerationsPage() -> impl IntoView {
    view! {
        <section class="page-content">
            <h2>"NixOS Generations"</h2>
            <p>"This page will compare booted/current NixOS generations across hosts and flag mismatches."</p>
        </section>
    }
}

