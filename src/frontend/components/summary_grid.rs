use leptos::prelude::*;

use crate::frontend::alerts::alert_summary_panel;
use crate::frontend::certificates::certificate_summary_panel;
use crate::frontend::components::summary_panel::{SummaryPanel, SummaryPanelData};
use crate::frontend::hosts::host_summary_panel;
use crate::frontend::models::{CertificateExpiry, FiringAlert, HostStatus};

#[component]
pub fn SummaryGrid(
    alerts: ReadSignal<Vec<FiringAlert>>,
    alerts_loaded: ReadSignal<bool>,
    hosts: ReadSignal<Vec<HostStatus>>,
    hosts_loaded: ReadSignal<bool>,
    certificates: ReadSignal<Vec<CertificateExpiry>>,
    certificates_loaded: ReadSignal<bool>,
) -> impl IntoView {
view! {
            <div class="summary-grid">
                    {move || {
                        let data = if alerts_loaded.get() {
                            alert_summary_panel(&alerts.get())
                        } else {
                            SummaryPanelData {
                                title: "Alerts",
                                empty_message: "No alerts firing.",
                                items: Vec::new(),
                            }
                        };

                        view! {
                            <SummaryPanel
                                loading=!alerts_loaded.get()
                                data=data
                            />
                        }.into_any()
                    }}
                    {move || {
                        let data = if hosts_loaded.get() {
                            host_summary_panel(&hosts.get())
                        } else {
                            SummaryPanelData {
                                title: "Hosts",
                                empty_message: "No hosts found.",
                                items: Vec::new(),
                            }
                        };

                        view! {
                            <SummaryPanel
                                loading=!hosts_loaded.get()
                                data=data
                            />
                        }.into_any()
                    }}
                    {move || {
                        let data = if certificates_loaded.get() {
                            certificate_summary_panel(&certificates.get())
                        } else {
                            SummaryPanelData {
                                title: "Certificates",
                                empty_message: "No certificate data found.",
                                items: Vec::new(),
                            }
                        };

                        view! {
                            <SummaryPanel
                                loading=!certificates_loaded.get()
                                data=data
                            />
                        }.into_any()
                    }}
            </div>

    }
}
