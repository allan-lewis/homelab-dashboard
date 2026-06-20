use serde::Deserialize;

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
