use gloo_timers::future::TimeoutFuture;
use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::frontend::hosts::fetch_hosts;
use crate::frontend::models::{HostState, HostStatus};

#[component]
pub fn HostsPage() -> impl IntoView {
    let (hosts, set_hosts) = signal(Vec::<HostStatus>::new());
    let (loaded, set_loaded) = signal(false);

    spawn_local(async move {
        loop {
            let loaded_hosts = fetch_hosts().await;

            set_hosts.set(loaded_hosts);
            set_loaded.set(true);

            TimeoutFuture::new(10_000).await;
        }
    });

    view! {
        <section class="page-content">
            <h2>"Hosts"</h2>

            {move || {
                if !loaded.get() {
                    view! { <p>"Loading hosts..."</p> }.into_any()
                } else {
                    view! {
                        <table class="status-table">
                            <thead>
                                <tr>
                                <th>"Hostname"</th>
                                <th>"Persona"</th>
                                <th>"IP Address"</th>
                                <th>"Status"</th>
                                </tr>
                            </thead>

                            <tbody>
                                {hosts
                                    .get()
                                    .into_iter()
                                    .map(|host| {
                                        let status_class = match host.status {
                                            HostState::Up => "status-pill up",
                                            HostState::Down => "status-pill down",
                                            HostState::Unknown => "status-pill unknown",
                                        };

                                        let status_label = match host.status {
                                            HostState::Up => "Up",
                                            HostState::Down => "Down",
                                            HostState::Unknown => "Unknown",
                                        };
                                        view! {
                                            <tr>
                                                <td>{host.hostname}</td>
                                                <td>{host.persona}</td>
                                                <td>{host.ip_address}</td>
                                                <td>
                                                    <span class=status_class>
                                                        {status_label}
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

