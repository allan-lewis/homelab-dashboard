use gloo_timers::future::TimeoutFuture;
use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::frontend::generations::fetch_generations;
use crate::frontend::models::NixosGeneration;

#[component]
pub fn GenerationsPage() -> impl IntoView {
    let (generations, set_generations) = signal(Vec::<NixosGeneration>::new());
    let (loaded, set_loaded) = signal(false);

    spawn_local(async move {
        loop {
            let loaded_generations = fetch_generations().await;

            set_generations.set(loaded_generations);
            set_loaded.set(true);

            TimeoutFuture::new(10_000).await;
        }
    });

    view! {
        <section class="page-content">
            <h2>"NixOS Generations"</h2>

            {move || {
                if !loaded.get() {
                    view! {
                        <p>"Loading generations..."</p>
                    }.into_any()
                } else if generations.get().is_empty() {
                    view! {
                        <p>"No generation data found."</p>
                    }.into_any()
                } else {
                    view! {
                        <table class="status-table">
                            <thead>
                                <tr>
                                    <th>"Instance"</th>
                                    <th>"Current Version"</th>
                                    <th>"Booted Generation"</th>
                                    <th>"Current Generation"</th>
                                    <th>"Latest Booted"</th>
                                </tr>
                            </thead>

                            <tbody>
                                {generations
                                    .get()
                                    .into_iter()
                                    .map(|generation| {
                                        let latest_booted = generation.booted_is_current;

                                        view! {
                                            <tr>
                                                <td>{generation.instance}</td>
                                                <td>{generation.current_version}</td>
                                                <td>{generation.booted_generation}</td>
                                                <td>{generation.current_generation}</td>
                                                <td>
                                                    <span class={
                                                        if latest_booted {
                                                            "status-pill up"
                                                        } else {
                                                            "status-pill warning"
                                                        }
                                                    }>
                                                        {if latest_booted { "Yes" } else { "No" }}
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
