use std::collections::HashMap;

use gloo_timers::future::TimeoutFuture;
use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::frontend::models::{LiveCheckStatus, SloCheckStatus};
use crate::frontend::slo::{fetch_live_status, fetch_slo_status, live_pill_class, live_pill_label, slo_pill_class};

#[component]
pub fn SloPage() -> impl IntoView {
    let (checks, set_checks) = signal(Vec::<SloCheckStatus>::new());
    let (loaded, set_loaded) = signal(false);
    let (live_statuses, set_live_statuses) = signal(Vec::<LiveCheckStatus>::new());

    spawn_local(async move {
        loop {
            let loaded_checks = fetch_slo_status().await;

            set_checks.set(loaded_checks);
            set_loaded.set(true);

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
        <section class="page-content">
            <h2>"SLOs"</h2>

            {move || {
                if !loaded.get() {
                    view! { <p>"Loading SLO status..."</p> }.into_any()
                } else if checks.get().is_empty() {
                    view! { <p>"No SLO checks configured yet."</p> }.into_any()
                } else {
                    let live_by_key: HashMap<String, _> = live_statuses
                        .get()
                        .into_iter()
                        .map(|status| (status.key, status.state))
                        .collect();

                    view! {
                        <table class="status-table">
                            <thead>
                                <tr>
                                    <th>"Name"</th>
                                    <th>"Description"</th>
                                    <th>"URL"</th>
                                    <th>"Spot Health"</th>
                                    <th>"SLO"</th>
                                </tr>
                            </thead>

                            <tbody>
                                {checks
                                    .get()
                                    .into_iter()
                                    .map(|check| {
                                        let live_state = live_by_key.get(&check.key).cloned();
                                        let pct_28d = check.pct_28d;

                                        view! {
                                            <tr>
                                                <td>{check.name}</td>
                                                <td>{check.slo_desc.unwrap_or_default()}</td>
                                                <td>
                                                    {check
                                                        .slo_url
                                                        .map(|url| {
                                                            let link_text = url.clone();
                                                            view! {
                                                                <a
                                                                    class="slo-check-link"
                                                                    href=url
                                                                    target="_blank"
                                                                    rel="noopener noreferrer"
                                                                >
                                                                    {link_text}
                                                                </a>
                                                            }
                                                        })}
                                                </td>
                                                <td>
                                                    {live_state
                                                        .map(|state| {
                                                            view! {
                                                                <span class=live_pill_class(&state)>
                                                                    {live_pill_label(&state)}
                                                                </span>
                                                            }
                                                        })}
                                                </td>
                                                <td>
                                                    <span class=slo_pill_class(pct_28d)>
                                                        {format!("{:.3}%", pct_28d * 100.0)}
                                                    </span>
                                                </td>
                                            </tr>
                                        }
                                    })
                                    .collect_view()}
                            </tbody>
                        </table>
                    }
                        .into_any()
                }
            }}
        </section>
    }
}
