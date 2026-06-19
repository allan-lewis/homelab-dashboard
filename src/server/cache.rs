use std::{future::Future, sync::Arc};

use tokio::{
    sync::RwLock,
    time::{sleep, Duration},
};

pub fn start_polling<T, F, Fut>(
    name: &'static str,
    prometheus_url: String,
    cache: Arc<RwLock<Vec<T>>>,
    fetch: F,
) where
    T: Send + Sync + 'static,
    F: Fn(String, reqwest::Client) -> Fut + Send + Sync + Copy + 'static,
    Fut: Future<Output = Result<Vec<T>, String>> + Send + 'static,
{
    tokio::spawn(async move {
        let client = reqwest::Client::new();

        loop {
            match fetch(prometheus_url.clone(), client.clone()).await {
                Ok(statuses) => {
                    println!("Updated {name} cache: {} entries", statuses.len());
                    *cache.write().await = statuses;
                }
                Err(err) => {
                    eprintln!("Failed to update {name} cache: {err}");
                }
            }

            sleep(Duration::from_secs(30)).await;
        }
    });
}
