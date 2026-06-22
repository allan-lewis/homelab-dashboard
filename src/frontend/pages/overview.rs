use gloo_net::http::Request;
use gloo_timers::future::TimeoutFuture;
use js_sys::Date;
use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::frontend::alerts::fetch_alerts;
use crate::frontend::components::overview_info_card::OverviewInfoCard;
use crate::frontend::components::overview_status_card::OverviewStatusCard;
use crate::frontend::components::summary_grid::SummaryGrid;
use crate::frontend::certificates::CertificateSummary;
use crate::frontend::certificates::{fetch_certificates};
use crate::frontend::hosts::fetch_hosts;
use crate::frontend::models::{CertificateExpiry, FiringAlert, HostStatus};

fn overview_is_healthy(
    critical_alerts: usize,
    warning_alerts: usize,
    down_hosts: usize,
    certificate_summary: &CertificateSummary,
) -> bool {
    critical_alerts == 0
        && warning_alerts == 0
        && down_hosts == 0
        && certificate_summary.critical_count == 0
        && certificate_summary.warning_count == 0
}

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

    let (certificates, set_certificates) = signal(Vec::<CertificateExpiry>::new());
    let (certificates_loaded, set_certificates_loaded) = signal(false);

    let (last_updated, set_last_updated) = signal(None::<String>);

    spawn_local(async move {
        loop {
            let loaded_certificates = fetch_certificates().await;

            set_certificates.set(loaded_certificates);
            set_certificates_loaded.set(true);
            set_last_updated.set(Some(current_utc_time_string()));

            TimeoutFuture::new(10_000).await;
        }
    });

    spawn_local(async move {
        loop {
            let loaded_alerts = fetch_alerts().await;

            set_alerts.set(loaded_alerts);
            set_alerts_loaded.set(true);
            set_last_updated.set(Some(current_utc_time_string()));

            TimeoutFuture::new(10_000).await;
        }
    });

    spawn_local(async move {
        loop {
            let loaded_hosts = fetch_hosts().await;

            set_hosts.set(loaded_hosts);
            set_hosts_loaded.set(true);
            set_last_updated.set(Some(current_utc_time_string()));

            TimeoutFuture::new(10_000).await;
        }
    });

    view! {
        <section class="page-content">
            <h2>"Overview"</h2>

            <div class="overview-top-grid">
                <OverviewStatusCard />
                <OverviewInfoCard />
            </div>

            <SummaryGrid
                alerts=alerts
                alerts_loaded=alerts_loaded
                hosts=hosts
                hosts_loaded=hosts_loaded
                certificates=certificates
                certificates_loaded=certificates_loaded
            />
        </section>
    }
}
