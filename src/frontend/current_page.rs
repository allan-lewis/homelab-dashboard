use leptos::prelude::*;

use crate::frontend::pages::generations::GenerationsPage;
use crate::frontend::pages::hosts::HostsPage;
use crate::frontend::pages::overview::OverviewPage;
use crate::frontend::pages::uptime::UptimePage;
use crate::frontend::routing::Page;

#[component]
pub fn CurrentPage(current_page: ReadSignal<Page>, name: String) -> impl IntoView {
    view! {
        {move || {
            match current_page.get() {
                Page::Overview => view! { <OverviewPage name=name.clone() /> }.into_any(),
                Page::Hosts => view! { <HostsPage /> }.into_any(),
                Page::Generations => view! { <GenerationsPage /> }.into_any(),
                Page::Uptime => view! { <UptimePage /> }.into_any(),
            }
        }}
    }
}

