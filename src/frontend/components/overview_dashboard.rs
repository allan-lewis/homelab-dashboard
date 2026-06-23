use gloo_timers::future::TimeoutFuture;
use js_sys::Date;
use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::frontend::components::overview_info_card::OverviewInfoCard;
use crate::frontend::components::overview_status_card::OverviewStatusCard;
use crate::frontend::components::summary_grid::SummaryGrid;
use crate::frontend::models::{CertificateExpiry, FiringAlert, HostStatus, TaskStatus};
use crate::frontend::tasks::{fetch_tasks, task_status_lines, task_summary_panel};
use crate::frontend::components::summary_panel::SummaryPanelState;
use crate::frontend::alerts::{
    alert_info_lines, alert_status_lines, alert_summary_panel, fetch_alerts,
};
use crate::frontend::certificates::{
    certificate_info_lines, certificate_status_lines, certificate_summary_panel, fetch_certificates,
};
use crate::frontend::hosts::{
    fetch_hosts, host_info_lines, host_status_lines, host_summary_panel,
};

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
pub fn OverviewDashboard() -> impl IntoView {
    let (certificates, set_certificates) = signal(Vec::<CertificateExpiry>::new());
    let (certificates_loaded, set_certificates_loaded) = signal(false);

    let (alerts, set_alerts) = signal(Vec::<FiringAlert>::new());
    let (alerts_loaded, set_alerts_loaded) = signal(false);

    let (hosts, set_hosts) = signal(Vec::<HostStatus>::new());
    let (hosts_loaded, set_hosts_loaded) = signal(false);

    let (tasks, set_tasks) = signal(Vec::<TaskStatus>::new());
    let (tasks_loaded, set_tasks_loaded) = signal(false);

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

    spawn_local(async move {
        loop {
            let loaded_tasks = fetch_tasks().await;

            set_tasks.set(loaded_tasks);
            set_tasks_loaded.set(true);
            set_last_updated.set(Some(current_utc_time_string()));

            TimeoutFuture::new(10_000).await;
        }
    });

    view! {
        <div class="overview-top-grid">
            {move || {
                let mut status_lines = Vec::new();

                if alerts_loaded.get() {
                    status_lines.extend(alert_status_lines(&alerts.get()));
                }

                if hosts_loaded.get() {
                    status_lines.extend(host_status_lines(&hosts.get()));
                }

                if certificates_loaded.get() {
                    status_lines.extend(certificate_status_lines(&certificates.get()));
                }

                if tasks_loaded.get() {
                    status_lines.extend(task_status_lines(&tasks.get()));
                }

                let loading = !alerts_loaded.get()
                    || !hosts_loaded.get()
                    || !certificates_loaded.get()
                    || !tasks_loaded.get();

                view! {
                    <OverviewStatusCard
                        loading=loading
                        lines=status_lines
                    />
                }
            }}
            {move || {
                let mut info_lines = Vec::new();

                if alerts_loaded.get() {
                    info_lines.extend(alert_info_lines(&alerts.get()));
                }

                if hosts_loaded.get() {
                    info_lines.extend(host_info_lines(&hosts.get()));
                }

                if certificates_loaded.get() {
                    info_lines.extend(certificate_info_lines(&certificates.get()));
                }

                let loading = !alerts_loaded.get()
                    || !hosts_loaded.get()
                    || !certificates_loaded.get();

                view! {
                    <OverviewInfoCard
                        loading=loading
                        lines=info_lines
                        last_updated=last_updated
                    />
                }
            }}
        </div>

        {move || {
            let panels = vec![
                SummaryPanelState {
                    loading: !alerts_loaded.get(),
                    data: alert_summary_panel(&alerts.get()),
                },
                SummaryPanelState {
                    loading: !hosts_loaded.get(),
                    data: host_summary_panel(&hosts.get()),
                },
                SummaryPanelState {
                    loading: !certificates_loaded.get(),
                    data: certificate_summary_panel(&certificates.get()),
                },
                SummaryPanelState {
                    loading: !tasks_loaded.get(),
                    data: task_summary_panel(&tasks.get()),
                },
            ];

            view! {
                <SummaryGrid panels=panels />
            }
        }}
    }
}
