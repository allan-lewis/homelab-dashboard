use std::{future::Future, sync::Arc};
use tracing::{error, info};
use tokio::{
    sync::RwLock,
    time::{Duration, sleep},
};
use super::{
    metrics::fetch_firing_alerts,
    models::FiringAlert,
};

pub fn start_firing_alerts_polling(
    prometheus_url: String,
    alertmanager_url: String,
    cache: Arc<RwLock<Vec<FiringAlert>>>,
) {
    start_polling(
        "firing_alerts",
        prometheus_url,
        cache,
        move |prometheus_url, client| {
            let alertmanager_url = alertmanager_url.clone();

            async move {
                fetch_firing_alerts(
                    prometheus_url,
                    alertmanager_url,
                    client,
                )
                .await
            }
        },
    );
}

pub fn start_polling<T, F, Fut>(
    name: &'static str,
    prometheus_url: String,
    cache: Arc<RwLock<Vec<T>>>,
    fetch: F,
) where
    T: Send + Sync + 'static,
    F: Fn(String, reqwest::Client) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<Vec<T>, String>> + Send + 'static,
{
    tokio::spawn(async move {
        let client = reqwest::Client::new();

        loop {
            match fetch(prometheus_url.clone(), client.clone()).await {
                Ok(statuses) => {
                    let current_count = statuses.len();

                    let mut cache_guard = cache.write().await;
                    let previous_count = cache_guard.len();
                    let delta = current_count as isize - previous_count as isize;

                    *cache_guard = statuses;

                    info!(
                        cache = name,
                        previous_count, current_count, delta, "cache updated"
                    );
                }
                Err(err) => {
                    error!(cache = name, %err, "cache update failed");
                }
            }

            sleep(Duration::from_secs(30)).await;
        }
    });
}
