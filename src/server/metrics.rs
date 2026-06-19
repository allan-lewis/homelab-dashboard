use super::{
    models::{GatusHostStatus, HostUpStatus, PrometheusQueryResponse},
    util::hostname_from_name,
};

pub async fn fetch_gatus_hosts(
    prometheus_url: String,
    client: reqwest::Client,
) -> Result<Vec<GatusHostStatus>, String> {
    let response = client
        .get(format!("{}/api/v1/query", prometheus_url))
        .query(&[("query", r#"gatus_results_endpoint_success{group="Hosts"}"#)])
        .send()
        .await
        .map_err(|err| format!("failed to query Prometheus: {err}"))?;

    let prometheus_response = response
        .json::<PrometheusQueryResponse>()
        .await
        .map_err(|err| format!("failed to parse Prometheus response: {err}"))?;

    let mut statuses = prometheus_response
        .data
        .result
        .into_iter()
        .map(|result| {
            let name = result
                .metric
                .get("name")
                .cloned()
                .unwrap_or_else(|| "unknown".to_string());

            let instance = hostname_from_name(&name);

            let target = result.metric.get("target").cloned().unwrap_or_default();

            GatusHostStatus {
                instance,
                name,
                target,
                timestamp: result.value.0,
                up: result.value.1 == "1",
            }
        })
        .collect::<Vec<_>>();

    statuses.sort_by(|a, b| a.instance.cmp(&b.instance));

    Ok(statuses)
}

pub async fn fetch_prometheus_up(
    prometheus_url: String,
    client: reqwest::Client,
) -> Result<Vec<HostUpStatus>, String> {
    let response = client
        .get(format!("{}/api/v1/query", prometheus_url))
        .query(&[("query", "up")])
        .send()
        .await
        .map_err(|err| format!("failed to query Prometheus: {err}"))?;

    let prometheus_response = response
        .json::<PrometheusQueryResponse>()
        .await
        .map_err(|err| format!("failed to parse Prometheus response: {err}"))?;

    let mut statuses = prometheus_response
        .data
        .result
        .into_iter()
        .map(|result| {
            let instance = result
                .metric
                .get("instance")
                .cloned()
                .unwrap_or_else(|| "unknown".to_string());

            let job = result
                .metric
                .get("job")
                .cloned()
                .unwrap_or_else(|| "unknown".to_string());

            let target = result.metric.get("target").cloned().unwrap_or_default();

            HostUpStatus {
                instance,
                job,
                target,
                timestamp: result.value.0,
                up: result.value.1 == "1",
            }
        })
        .collect::<Vec<_>>();

    statuses.sort_by(|a, b| {
        a.instance
            .cmp(&b.instance)
            .then_with(|| a.job.cmp(&b.job))
            .then_with(|| a.target.cmp(&b.target))
    });

    Ok(statuses)
}
