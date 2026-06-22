use gloo_net::http::Request;
use gloo_timers::future::TimeoutFuture;
use js_sys::Date;
use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::frontend::components::summary_line::SummaryLine;
use crate::frontend::models::{FiringAlert, HostState, HostStatus};

fn current_utc_time_string() -> String {
    let now = Date::new_0();

    let weekdays = [
        "Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday",
    ];

    let months = [
        "January", "February", "March", "April", "May", "June", "July", "August",
        "September", "October", "November", "December",
    ];

    format!(
        "{}, {} {} at {:02}:{:02}:{:02} UTC",
        weekdays[now.get_utc_day() as usize],
        months[now.get_utc_month() as usize],
        now.get_utc_date(),
        now.get_utc_hours(),
        now.get_utc_minutes(),
        now.get_utc_seconds(),
    )
}

#[component]
pub fn OverviewPage() -> impl IntoView {
    let (alerts, set_alerts) = signal(Vec::<FiringAlert>::new());
    let (alerts_loaded, set_alerts_loaded) = signal(false);

    let (hosts, set_hosts) = signal(Vec::<HostStatus>::new());
    let (hosts_loaded, set_hosts_loaded) = signal(false);

    let (last_updated, set_last_updated) = signal(None::<String>);

    spawn_local(async move {
        loop {
            let loaded_alerts = match Request::get("/api/alerts").send().await {
                Ok(response) => response.json::<Vec<FiringAlert>>().await.unwrap_or_default(),
                Err(_) => Vec::new(),
            };

            set_alerts.set(loaded_alerts);
            set_alerts_loaded.set(true);
            set_last_updated.set(Some(current_utc_time_string()));

            TimeoutFuture::new(10_000).await;
        }
    });

    spawn_local(async move {
        loop {
            let loaded_hosts = match Request::get("/api/hosts").send().await {
                Ok(response) => response.json::<Vec<HostStatus>>().await.unwrap_or_default(),
                Err(_) => Vec::new(),
            };

            set_hosts.set(loaded_hosts);
            set_hosts_loaded.set(true);
            set_last_updated.set(Some(current_utc_time_string()));

            TimeoutFuture::new(10_000).await;
        }
    });

    view! {
        <section class="page-content">
            <h2>"Overview"</h2>

            <div class="overview-hero">
                {move || {
                    if !alerts_loaded.get() || !hosts_loaded.get() {
                        view! {
                            <p class="overview-summary">
                                "Loading dashboard summary..."
                            </p>
                        }.into_any()
                    } else {
                        let alerts = alerts.get();
                        let hosts = hosts.get();

                        let critical_alerts = alerts
                            .iter()
                            .filter(|alert| alert.severity == "critical")
                            .count();

                        let warning_alerts = alerts
                            .iter()
                            .filter(|alert| alert.severity == "warning")
                            .count();

                        let down_hosts = hosts
                            .iter()
                            .filter(|host| matches!(host.status, HostState::Down))
                            .count();
if critical_alerts == 0 && warning_alerts == 0 && down_hosts == 0 {
    view! {
        <p class="overview-summary">
            "All monitored systems look healthy."
        </p>
    }.into_any()
} else {
    view! {
        <div>
            <div class="overview-status-list">
                {if critical_alerts > 0 || warning_alerts > 0 {
                    view! {
                        <div class="overview-status-line">
                            {critical_alerts}
                            " critical alerts firing and "
                            {warning_alerts}
                            " warning alerts firing."
                        </div>
                    }.into_any()
                } else {
                    view! {}.into_any()
                }}

                        <div class="overview-status-line">
                            {down_hosts}
                            " hosts down."
                        </div>
            </div>
        </div>
    }.into_any()
}
                    }
                }}

                <p class="overview-subsummary">
                    {move || {
                        if !alerts_loaded.get() || !hosts_loaded.get() {
                            "Waiting for data...".to_string()
                        } else {
                            let alerts = alerts.get();
                            let hosts = hosts.get();

                            let info_alerts = alerts
                                .iter()
                                .filter(|alert| alert.severity == "info")
                                .count();

                            match last_updated.get() {
                                Some(updated) => format!(
                                    "{} info alerts active. {} hosts reporting. Last updated {}.",
                                    info_alerts,
                                    hosts.len(),
                                    updated,
                                ),
                                None => format!(
                                    "{} info alerts active. {} hosts reporting.",
                                    info_alerts,
                                    hosts.len(),
                                ),
                            }
                        }
                    }}
                </p>
            </div>

            <div class="summary-grid">
                <section class="summary-panel">
                    <h3>"Alerts"</h3>

                    {move || {
                        if !alerts_loaded.get() {
                            view! { <p>"Loading alerts..."</p> }.into_any()
                        } else {
                            let alerts = alerts.get();

                            let critical_count = alerts
                                .iter()
                                .filter(|alert| alert.severity == "critical")
                                .count();

                            let warning_count = alerts
                                .iter()
                                .filter(|alert| alert.severity == "warning")
                                .count();

                            let info_count = alerts
                                .iter()
                                .filter(|alert| alert.severity == "info")
                                .count();

                            if critical_count == 0 && warning_count == 0 && info_count == 0 {
                                view! {
                                    <p>"No alerts firing."</p>
                                }.into_any()
                            } else {
                                view! {
                                    <div class="summary-list">
                                        {if critical_count > 0 {
                                            view! {
                                                <SummaryLine
                                                    label="Critical alerts firing"
                                                    count=critical_count
                                                    pill_class="status-pill down"
                                                />
                                            }.into_any()
                                        } else {
                                            view! {}.into_any()
                                        }}
                                        {if warning_count > 0 {
                                            view! {
                                                <SummaryLine
                                                    label="Warning alerts firing"
                                                    count=warning_count
                                                    pill_class="status-pill warning"
                                                />
                                            }.into_any()
                                        } else {
                                            view! {}.into_any()
                                        }}
                                        {if info_count > 0 {
                                            view! {
                                                <SummaryLine
                                                    label="Info alerts firing"
                                                    count=info_count
                                                    pill_class="status-pill info"
                                                />
                                            }.into_any()
                                        } else {
                                            view! {}.into_any()
                                        }}
                                    </div>
                                }.into_any()
                            }
                        }
                    }}
                </section>

                <section class="summary-panel">
                    <h3>"Hosts"</h3>

                    {move || {
                        if !hosts_loaded.get() {
                            view! { <p>"Loading hosts..."</p> }.into_any()
                        } else {
                            let hosts = hosts.get();

                            let down_count = hosts
                                .iter()
                                .filter(|host| matches!(host.status, HostState::Down))
                                .count();

                            let unknown_count = hosts
                                .iter()
                                .filter(|host| matches!(host.status, HostState::Unknown))
                                .count();

                            let up_count = hosts
                                .iter()
                                .filter(|host| matches!(host.status, HostState::Up))
                                .count();

                            if down_count == 0 && unknown_count == 0 && up_count == 0 {
                                view! {
                                    <p>"No hosts found."</p>
                                }.into_any()
                            } else {
                                view! {
                                    <div class="summary-list">
                                        {if down_count > 0 {
                                            view! {
                                                <SummaryLine
                                                    label="Hosts down"
                                                    count=down_count
                                                    pill_class="status-pill down"
                                                />
                                            }.into_any()
                                        } else {
                                            view! {}.into_any()
                                        }}

                                        {if unknown_count > 0 {
                                            view! {
                                                <SummaryLine
                                                    label="Hosts unknown"
                                                    count=unknown_count
                                                    pill_class="status-pill unknown"
                                                />
                                            }.into_any()
                                        } else {
                                            view! {}.into_any()
                                        }}

                                        {if up_count > 0 {
                                            view! {
                                                <SummaryLine
                                                    label="Hosts up"
                                                    count=up_count
                                                    pill_class="status-pill up"
                                                />
                                            }.into_any()
                                        } else {
                                            view! {}.into_any()
                                        }}
                                    </div>
                                }.into_any()
                            }
                        }
                    }}
                </section>
            </div>
        </section>
    }
}
