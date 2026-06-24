use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct PrometheusTarget {
    pub instance: String,
    pub job: String,
    pub target: String,
    pub up: bool,
}

#[derive(Clone, Debug, Deserialize)]
pub struct NixosGeneration {
    pub instance: String,
    pub booted_is_current: bool,
    pub booted_generation: String,
    pub current_generation: String,
    pub current_version: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct TaskStatus {
    pub instance: String,
    pub name: String,
    pub age_ratio: f64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct CertificateExpiry {
    pub name: String,
    pub group: String,
    pub cert_expiry_seconds: f64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct FiringAlert {
    pub alertname: String,
    pub rulegroup: String,
    pub severity: String,
    pub instance: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HostState {
    Up,
    Down,
    Unknown,
}

#[derive(Clone, Debug, Deserialize)]
pub struct HostStatus {
    pub hostname: String,
    pub persona: String,
    pub ip_address: String,
    pub status: HostState,
}

#[derive(Clone, Debug, Deserialize)]
pub struct User {
    pub name: String,
    pub email: String,
}

#[derive(Clone, Debug)]
pub enum AuthState {
    Loading,
    Anonymous,
    Authenticated(User),
}
