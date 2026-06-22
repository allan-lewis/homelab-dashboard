use gloo_timers::future::TimeoutFuture;
use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::frontend::certificates::{
    expiry_class, expiry_label, fetch_certificates,
};
use crate::frontend::models::CertificateExpiry;

#[component]
pub fn CertificatesPage() -> impl IntoView {
    let (certificates, set_certificates) = signal(Vec::<CertificateExpiry>::new());
    let (loaded, set_loaded) = signal(false);

    spawn_local(async move {
        loop {
            let loaded_certificates = fetch_certificates().await;

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
                                        let expiry_class = expiry_class(&certificate);
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
