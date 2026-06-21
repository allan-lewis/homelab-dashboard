use gloo_net::http::Request;
use gloo_timers::future::TimeoutFuture;
use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::frontend::models::FiringAlert;

#[component]
pub fn AlertsPage() -> impl IntoView {
    let (alerts, set_alerts) = signal(Vec::<FiringAlert>::new());
    let (loaded, set_loaded) = signal(false);

    spawn_local(async move {
        loop {
            let loaded_alerts = match Request::get("/api/alerts").send().await {
                Ok(response) => response.json::<Vec<FiringAlert>>().await.unwrap_or_default(),
                Err(_) => Vec::new(),
            };

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
                                </tr>
                            </thead>

                            <tbody>
                                {alerts
                                    .get()
                                    .into_iter()
                                    .map(|alert| {
                                        let severity_class = match alert.severity.as_str() {
                                            "critical" => "status-pill down",
                                            "info" => "status-pill unknown",
                                            _ => "status-pill unknown",
                                        };

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
