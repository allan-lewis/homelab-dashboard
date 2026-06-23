use gloo_timers::future::TimeoutFuture;
use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::frontend::models::TaskStatus;
use crate::frontend::tasks::{fetch_tasks, task_is_late};

#[component]
pub fn TasksPage() -> impl IntoView {
    let (tasks, set_tasks) = signal(Vec::<TaskStatus>::new());
    let (loaded, set_loaded) = signal(false);

    spawn_local(async move {
        loop {
            let loaded_tasks = fetch_tasks().await;

            set_tasks.set(loaded_tasks);
            set_loaded.set(true);

            TimeoutFuture::new(10_000).await;
        }
    });

    view! {
        <section class="page-content">
            <h2>"Tasks"</h2>

            {move || {
                if !loaded.get() {
                    view! {
                        <p>"Loading tasks..."</p>
                    }.into_any()
                } else if tasks.get().is_empty() {
                    view! {
                        <p>"No task data found."</p>
                    }.into_any()
                } else {
                    view! {
                        <table class="status-table">
                            <thead>
                                <tr>
                                    <th>"Instance"</th>
                                    <th>"Name"</th>
                                    <th>"Status"</th>
                                </tr>
                            </thead>

                            <tbody>
                                {tasks
                                    .get()
                                    .into_iter()
                                    .map(|task| {
                                        let late = task_is_late(&task);

                                        view! {
                                            <tr>
                                                <td>{task.instance}</td>
                                                <td>{task.name}</td>
                                                <td>
                                                    <span class={
                                                        if late {
                                                            "status-pill down"
                                                        } else {
                                                            "status-pill up"
                                                        }
                                                    }>
                                                        {if late { "Late" } else { "Ok" }}
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
