use gloo_net::http::Request;
use gloo_timers::future::TimeoutFuture;
use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::frontend::models::CertificateExpiry;

fn days_until_expiry(seconds: f64) -> i64 {
    (seconds / 86_400.0).floor() as i64
}

fn expiry_label(seconds: f64) -> String {
    let days = days_until_expiry(seconds);

    if days == 1 {
        "1 day".to_string()
    } else {
        format!("{days} days")
    }
}

fn expiry_class(seconds: f64) -> &'static str {
    let days = days_until_expiry(seconds);

    if days <= 14 {
        "status-pill down"
    } else if days <= 40 {
        "status-pill warning"
    } else {
        "status-pill up"
    }
}

#[component]
pub fn CertificatesPage() -> impl IntoView {
    let (certificates, set_certificates) = signal(Vec::<CertificateExpiry>::new());
    let (loaded, set_loaded) = signal(false);

    spawn_local(async move {
        loop {
            let mut loaded_certificates = match Request::get("/api/certificates").send().await {
                Ok(response) => response
                    .json::<Vec<CertificateExpiry>>()
                    .await
                    .unwrap_or_default(),
                Err(_) => Vec::new(),
            };

            loaded_certificates.sort_by(|a, b| {
                a.cert_expiry_seconds
                    .partial_cmp(&b.cert_expiry_seconds)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then_with(|| a.name.cmp(&b.name))
            });

            set_certificates.set(loaded_certificates);
            set_loaded.set(true);

            TimeoutFuture::new(10_000).await;
        }
    });

    view! {
        <section class="page-content">
            <h2>"Certificates"</h2>

            {move || {
                if !loaded.get() {
                    view! { <p>"Loading certificates..."</p> }.into_any()
                } else if certificates.get().is_empty() {
                    view! { <p>"No certificate expiry data found."</p> }.into_any()
                } else {
                    view! {
                        <table class="status-table">
                            <thead>
                                <tr>
                                    <th>"Name"</th>
                                    <th>"Group"</th>
                                    <th>"Expires In"</th>
                                </tr>
                            </thead>

                            <tbody>
                                {certificates
                                    .get()
                                    .into_iter()
                                    .map(|certificate| {
                                        let expiry_class = expiry_class(certificate.cert_expiry_seconds);
                                        let expiry_label = expiry_label(certificate.cert_expiry_seconds);

                                        view! {
                                            <tr>
                                                <td>{certificate.name}</td>
                                                <td>{certificate.group}</td>
                                                <td>
                                                    <span class=expiry_class>
                                                        {expiry_label}
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
