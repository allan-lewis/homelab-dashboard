use std::collections::HashMap;

use gloo_timers::future::TimeoutFuture;
use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::frontend::components::overview_dashboard::current_utc_time_string;
use crate::frontend::components::slo_check_row::SloCheckRow;
use crate::frontend::models::{LiveCheckStatus, SloCheckStatus};
use crate::frontend::slo::{fetch_live_status, fetch_slo_status};

#[component]
pub fn PublicStatusPage() -> impl IntoView {
    let (checks, set_checks) = signal(Vec::<SloCheckStatus>::new());
    let (loaded, set_loaded) = signal(false);
    let (last_updated, set_last_updated) = signal(None::<String>);
    let (live_statuses, set_live_statuses) = signal(Vec::<LiveCheckStatus>::new());

    spawn_local(async move {
        loop {
            let loaded_checks = fetch_slo_status().await;

            set_checks.set(loaded_checks);
            set_loaded.set(true);
            set_last_updated.set(Some(current_utc_time_string()));

            TimeoutFuture::new(60_000).await;
        }
    });

    spawn_local(async move {
        loop {
            let loaded_live_statuses = fetch_live_status().await;

            set_live_statuses.set(loaded_live_statuses);

            TimeoutFuture::new(30_000).await;
        }
    });

    view! {
        <main class="public-status-page">
            <p class="public-status-last-updated">
                {move || match last_updated.get() {
                    Some(updated) => format!("Last updated {updated}"),
                    None => "Last updated time unavailable".to_string(),
                }}
            </p>

            <div class="public-status-content">
                <h1>"Allan's Home Lab Status"</h1>
                <p class="public-status-subtitle">"28-day uptime for key services."</p>

                {move || {
                    if !loaded.get() {
                        view! { <p>"Loading status..."</p> }.into_any()
                    } else if checks.get().is_empty() {
                        view! { <p>"No status checks configured yet."</p> }.into_any()
                    } else {
                        let live_by_key: HashMap<String, _> = live_statuses
                            .get()
                            .into_iter()
                            .map(|status| (status.key, status.state))
                            .collect();

                        view! {
                            <div class="slo-check-list">
                                {checks
                                    .get()
                                    .into_iter()
                                    .map(|check| {
                                        let live_state = live_by_key.get(&check.key).cloned();

                                        view! { <SloCheckRow check=check live_state=live_state /> }
                                    })
                                    .collect_view()}
                            </div>
                        }
                            .into_any()
                    }
                }}
            </div>
        </main>
    }
}
