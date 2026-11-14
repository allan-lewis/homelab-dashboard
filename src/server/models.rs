use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct AlertmanagerAlert {
    pub labels: HashMap<String, String>,
    pub status: AlertmanagerAlertStatus,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlertmanagerAlertStatus {
    pub silenced_by: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlertmanagerSilence {
    pub id: String,
    pub ends_at: String,
    pub status: AlertmanagerSilenceStatus,
}

#[derive(Debug, Deserialize)]
pub struct AlertmanagerSilenceStatus {
    pub state: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct PublicIpStatus {
    pub instance: String,
    pub target: String,
    pub public_ip: String,
    pub timestamp: f64,
}

#[derive(Clone, Debug, Serialize)]
pub struct NixosGeneration {
    pub instance: String,
    pub booted_is_current: bool,
    pub booted_generation: String,
    pub current_generation: String,
    pub booted_version: String,
    pub current_version: String,
    pub booted_system: String,
    pub current_system: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct HomelabTask {
    pub instance: String,
    pub name: String,
    pub age_ratio: f64,
}

#[derive(Clone, Debug, Serialize)]
pub struct CertificateExpiry {
    pub key: String,
    pub name: String,
    pub group: String,
    pub instance: String,
    pub target: String,
    pub cert_expiry_seconds: f64,
}

#[derive(Clone, Debug, Serialize)]
pub struct FiringAlert {
    pub key: String,
    pub alertname: String,
    pub name: String,
    pub rulegroup: String,
    pub severity: String,
    pub instance: String,
    pub timestamp: f64,
    pub silenced: bool,
    pub silence_count: usize,
    pub longest_silence_remaining_seconds: Option<f64>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum HostState {
    Up,
    Down,
}

#[derive(Clone, Debug, Serialize)]
pub struct HostStatus {
    pub hostname: String,
    pub persona: String,
    pub ip_address: String,
    pub public_ip: String,
    pub status: HostState,
    pub timestamp: f64,
}

#[derive(Debug, Deserialize)]
pub struct PrometheusQueryResponse {
    pub data: PrometheusData,
}

#[derive(Debug, Deserialize)]
pub struct PrometheusData {
    pub result: Vec<PrometheusResult>,
}

#[derive(Debug, Deserialize)]
pub struct PrometheusResult {
    pub metric: std::collections::HashMap<String, String>,
    pub value: (f64, String),
}

#[derive(Debug, Deserialize)]
pub struct PrometheusRangeQueryResponse {
    pub data: PrometheusRangeData,
}

#[derive(Debug, Deserialize)]
pub struct PrometheusRangeData {
    pub result: Vec<PrometheusRangeResult>,
}

#[derive(Debug, Deserialize)]
pub struct PrometheusRangeResult {
    pub metric: std::collections::HashMap<String, String>,
    pub values: Vec<(f64, String)>,
}

#[derive(Clone, Debug, Serialize)]
pub struct DailySloStatus {
    pub date: String,
    pub pct: Option<f64>,
}

#[derive(Clone, Debug, Serialize)]
pub struct SloCheckStatus {
    pub key: String,
    pub name: String,
    pub group: String,
    pub pct_28d: f64,
    pub daily: Vec<DailySloStatus>,
    pub slo_desc: Option<String>,
    pub slo_url: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum LiveState {
    Healthy,
    Unstable,
    Down,
}

#[derive(Clone, Debug, Serialize)]
pub struct LiveCheckStatus {
    pub key: String,
    pub state: LiveState,
}

#[derive(Clone, Debug, Serialize)]
pub struct GatusHostStatus {
    pub instance: String,
    pub name: String,
    pub target: String,
    pub timestamp: f64,
    pub up: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct HostUpStatus {
    pub instance: String,
    pub job: String,
    pub target: String,
    pub timestamp: f64,
    pub up: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct User {
    pub name: String,
    pub email: String,
}

#[derive(Debug, Deserialize)]
pub struct AuthCallbackParams {
    pub code: String,
    pub state: String,
}
