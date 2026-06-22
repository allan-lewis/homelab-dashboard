use gloo_net::http::Request;

use crate::frontend::components::summary_panel::{SummaryPanelData, SummaryPanelItem};
use crate::frontend::models::CertificateExpiry;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CertificateState {
    Critical,
    Warning,
    Healthy,
}

#[derive(Clone, Debug)]
pub struct CertificateSummary {
    pub critical_count: usize,
    pub warning_count: usize,
    pub healthy_count: usize,
    pub total_count: usize,
    pub next_expiry_days: Option<i64>,
}

pub async fn fetch_certificates() -> Vec<CertificateExpiry> {
    let mut certificates = match Request::get("/api/certificates").send().await {
        Ok(response) => response
            .json::<Vec<CertificateExpiry>>()
            .await
            .unwrap_or_default(),
        Err(_) => Vec::new(),
    };

    sort_certificates_by_expiry(&mut certificates);

    certificates
}

pub fn days_until_expiry(seconds: f64) -> i64 {
    (seconds / 86_400.0).floor() as i64
}

pub fn certificate_state(certificate: &CertificateExpiry) -> CertificateState {
    let days = days_until_expiry(certificate.cert_expiry_seconds);

    if days <= 40 {
        CertificateState::Critical
    } else if days <= 60 {
        CertificateState::Warning
    } else {
        CertificateState::Healthy
    }
}

pub fn expiry_label(seconds: f64) -> String {
    let days = days_until_expiry(seconds);

    if days == 1 {
        "1 day".to_string()
    } else {
        format!("{days} days")
    }
}

pub fn expiry_class(certificate: &CertificateExpiry) -> &'static str {
    match certificate_state(certificate) {
        CertificateState::Critical => "status-pill down",
        CertificateState::Warning => "status-pill warning",
        CertificateState::Healthy => "status-pill up",
    }
}

pub fn sort_certificates_by_expiry(certificates: &mut [CertificateExpiry]) {
    certificates.sort_by(|a, b| {
        a.cert_expiry_seconds
            .partial_cmp(&b.cert_expiry_seconds)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.name.cmp(&b.name))
    });
}

pub fn summarize_certificates(certificates: &[CertificateExpiry]) -> CertificateSummary {
    let critical_count = certificates
        .iter()
        .filter(|certificate| certificate_state(certificate) == CertificateState::Critical)
        .count();

    let warning_count = certificates
        .iter()
        .filter(|certificate| certificate_state(certificate) == CertificateState::Warning)
        .count();

    let healthy_count = certificates
        .iter()
        .filter(|certificate| certificate_state(certificate) == CertificateState::Healthy)
        .count();

    let next_expiry_days = certificates
        .iter()
        .map(|certificate| days_until_expiry(certificate.cert_expiry_seconds))
        .min();

    CertificateSummary {
        critical_count,
        warning_count,
        healthy_count,
        total_count: certificates.len(),
        next_expiry_days,
    }
}

pub fn certificate_summary_panel(certificates: &[CertificateExpiry]) -> SummaryPanelData {
    let summary = summarize_certificates(certificates);

    let mut items = Vec::new();

    if summary.critical_count > 0 {
        items.push(SummaryPanelItem {
            label: "Certificates critical",
            count: summary.critical_count,
            pill_class: "status-pill down",
        });
    }

    if summary.warning_count > 0 {
        items.push(SummaryPanelItem {
            label: "Certificates warning",
            count: summary.warning_count,
            pill_class: "status-pill warning",
        });
    }

    if summary.healthy_count > 0 {
        items.push(SummaryPanelItem {
            label: "Certificates healthy",
            count: summary.healthy_count,
            pill_class: "status-pill up",
        });
    }

    SummaryPanelData {
        title: "Certificates",
        empty_message: "No certificate data found.",
        items,
    }
}
