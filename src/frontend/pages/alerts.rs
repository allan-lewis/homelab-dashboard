use gloo_timers::future::TimeoutFuture;
use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::frontend::alerts::{fetch_alerts, silence_remaining_label};
use crate::frontend::models::FiringAlert;

#[component]
pub fn AlertsPage() -> impl IntoView {
    let (alerts, set_alerts) = signal(Vec::<FiringAlert>::new());
    let (loaded, set_loaded) = signal(false);

    spawn_local(async move {
        loop {
            let loaded_alerts = fetch_alerts().await;

            set_alerts.set(loaded_alerts);
            set_loaded.set(true);

            TimeoutFuture::new(10_000).await;
        }
    });

    view! {
        <section class="page-content">
            <h2>"Alerts"</h2>

            {move || {
                if !loaded.get() {
                    view! { <p>"Loading alerts..."</p> }.into_any()
                } else if alerts.get().is_empty() {
                    view! { <p>"No firing alerts."</p> }.into_any()
                } else {
                    view! {
                        <table class="status-table">
                            <thead>
                                <tr>
                                    <th>"Alert"</th>
                                    <th>"Rule Group"</th>
                                    <th>"Instance"</th>
                                    <th>"Severity"</th>
                                    <th>"Silenced"</th>
                                </tr>
                            </thead>

                            <tbody>
                                {alerts
                                    .get()
                                    .into_iter()
                                    .map(|alert| {
                                        let severity_class = if alert.silenced {
                                            "status-pill silenced"
                                        } else {
                                            match alert.severity.as_str() {
                                                "critical" => "status-pill down",
                                                "warning" => "status-pill warning",
                                                "info" => "status-pill info",
                                                _ => "status-pill unknown",
                                            }
                                        };

                                        let silence_label =
                                            silence_remaining_label(alert.longest_silence_remaining_seconds);

                                        view! {
                                            <tr>
                                                <td>{alert.alertname}</td>
                                                <td>{alert.rulegroup}</td>
                                                <td>{alert.instance}</td>
                                                <td>
                                                    <span class=severity_class>
                                                        {alert.severity}
                                                    </span>
                                                </td>
                                                <td>{silence_label}</td>

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
