#[derive(Clone, Debug)]
pub struct AppConfig {
    pub auth: AuthConfig,
    pub prometheus: PrometheusConfig,
}

#[derive(Clone, Debug)]
pub struct AuthConfig {
    pub client_id: String,
    pub client_secret: String,
    pub issuer_url: String,
    pub redirect_uri: String,
    pub logout_uri: String,
    pub post_logout_redirect_uri: String,
}

#[derive(Clone, Debug)]
pub struct PrometheusConfig {
    pub url: String,
}

impl AppConfig {
    pub fn from_env() -> Self {
        Self {
            auth: AuthConfig {
                client_id: std::env::var("HOMELAB_DASHBOARD_AUTHENTIK_CLIENT_ID")
                    .expect("missing HOMELAB_DASHBOARD_AUTHENTIK_CLIENT_ID"),
                client_secret: std::env::var("HOMELAB_DASHBOARD_AUTHENTIK_CLIENT_SECRET")
                    .expect("missing HOMELAB_DASHBOARD_AUTHENTIK_CLIENT_SECRET"),
                issuer_url: std::env::var("HOMELAB_DASHBOARD_AUTHENTIK_ISSUER_URL")
                    .expect("missing HOMELAB_DASHBOARD_AUTHENTIK_ISSUER_URL"),
                redirect_uri: std::env::var("HOMELAB_DASHBOARD_AUTHENTIK_REDIRECT_URI")
                    .expect("missing HOMELAB_DASHBOARD_AUTHENTIK_REDIRECT_URI"),
                logout_uri: std::env::var("HOMELAB_DASHBOARD_AUTHENTIK_LOGOUT_URI")
                    .expect("missing HOMELAB_DASHBOARD_AUTHENTIK_LOGOUT_URI"),
                post_logout_redirect_uri: std::env::var(
                    "HOMELAB_DASHBOARD_POST_LOGOUT_REDIRECT_URI",
                )
                .expect("missing HOMELAB_DASHBOARD_POST_LOGOUT_REDIRECT_URI"),
            },
            prometheus: PrometheusConfig {
                url: std::env::var("HOMELAB_DASHBOARD_PROMETHEUS_URL")
                    .expect("missing HOMELAB_DASHBOARD_PROMETHEUS_URL"),
            },
        }
    }
}

pub fn redacted(value: &str) -> &'static str {
    if value.is_empty() {
        "<empty>"
    } else {
        "<redacted>"
    }
}
