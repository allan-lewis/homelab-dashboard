use serde::{Deserialize, Serialize};

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
