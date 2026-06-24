use gloo_timers::future::TimeoutFuture;
use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::frontend::models::PrometheusTarget;
use crate::frontend::up::{fetch_targets, target_is_down};

#[component]
pub fn UpPage() -> impl IntoView {
    let (targets, set_targets) = signal(Vec::<PrometheusTarget>::new());
    let (loaded, set_loaded) = signal(false);

    spawn_local(async move {
        loop {
            let loaded_targets = fetch_targets().await;

            set_targets.set(loaded_targets);
            set_loaded.set(true);

            TimeoutFuture::new(10_000).await;
        }
    });

    view! {
        <section class="page-content">
            <h2>"Prometheus Targets"</h2>

            {move || {
                if !loaded.get() {
                    view! {
                        <p>"Loading targets..."</p>
                    }.into_any()
                } else if targets.get().is_empty() {
                    view! {
                        <p>"No targets found."</p>
                    }.into_any()
                } else {
                    view! {
                        <table class="status-table">
                            <thead>
                                <tr>
                                    <th>"Instance"</th>
                                    <th>"Job"</th>
                                    <th>"Target"</th>
                                    <th>"Up"</th>
                                </tr>
                            </thead>

                            <tbody>
                                {targets
                                    .get()
                                    .into_iter()
                                    .map(|target| {
                                        let down = target_is_down(&target);

                                        view! {
                                            <tr>
                                                <td>{target.instance}</td>
                                                <td>{target.job}</td>
                                                <td>{target.target}</td>
                                                <td>
                                                    <span class={
                                                        if down {
                                                            "status-pill down"
                                                        } else {
                                                            "status-pill up"
                                                        }
                                                    }>
                                                        {if down { "No" } else { "Yes" }}
                                                    </span>
                                                </td>
                                            </tr>
                                        }
                                    })
                                    .collect_view()}
                            </tbody>
                        </table>
                    }.into_any()
                }
            }}
        </section>
    }
}
