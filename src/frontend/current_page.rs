use leptos::prelude::*;

use crate::frontend::pages::alerts::AlertsPage;
use crate::frontend::pages::certificates::CertificatesPage;
use crate::frontend::pages::generations::GenerationsPage;
use crate::frontend::pages::hosts::HostsPage;
use crate::frontend::pages::overview::OverviewPage;
use crate::frontend::pages::tasks::TasksPage;
use crate::frontend::pages::up::UpPage;
use crate::frontend::routing::Page;

#[component]
pub fn CurrentPage(current_page: ReadSignal<Page>) -> impl IntoView {
    view! {
        {move || {
            match current_page.get() {
                Page::Overview => view! { <OverviewPage /> }.into_any(),
                Page::Alerts => view! { <AlertsPage /> }.into_any(),
                Page::Certificates => view! { <CertificatesPage /> }.into_any(),
                Page::Hosts => view! { <HostsPage /> }.into_any(),
                Page::Tasks => view! { <TasksPage /> }.into_any(),
                Page::Generations => view! { <GenerationsPage /> }.into_any(),
                Page::Up => view! { <UpPage /> }.into_any(),
            }
        }}
    }
}

