use std::collections::HashMap;

use chrono::{DateTime, Utc};

use super::{
    models::{AlertmanagerAlert, AlertmanagerSilence, CertificateExpiry, FiringAlert, GatusHostStatus, HomelabTask, HostUpStatus, NixosGeneration, PrometheusQueryResponse, PublicIpStatus},
    util::{friendly_name, hostname_from_name},
};

pub async fn fetch_nixos_generations(
    prometheus_url: String,
    client: reqwest::Client,
) -> Result<Vec<NixosGeneration>, String> {
    let response = client
        .get(format!("{}/api/v1/query", prometheus_url))
        .query(&[("query", "nixos_system_info")])
        .send()
        .await
        .map_err(|err| format!("failed to query Prometheus: {err}"))?;

    let prometheus_response = response
        .json::<PrometheusQueryResponse>()
        .await
        .map_err(|err| format!("failed to parse Prometheus response: {err}"))?;

    let mut generations = prometheus_response
        .data
        .result
        .into_iter()
        .map(|result| NixosGeneration {
            instance: result.metric.get("instance").cloned().unwrap_or_default(),
            booted_is_current: result
                .metric
                .get("booted_is_current")
                .map(|value| value == "true")
                .unwrap_or(false),
            booted_generation: result
                .metric
                .get("booted_generation")
                .cloned()
                .unwrap_or_default(),
            current_generation: result
                .metric
                .get("current_generation")
                .cloned()
                .unwrap_or_default(),
            booted_version: result
                .metric
                .get("booted_version")
                .cloned()
                .unwrap_or_default(),
            current_version: result
                .metric
                .get("current_version")
                .cloned()
                .unwrap_or_default(),
            booted_system: result
                .metric
                .get("booted_system")
                .cloned()
                .unwrap_or_default(),
            current_system: result
                .metric
                .get("current_system")
                .cloned()
                .unwrap_or_default(),
        })
        .collect::<Vec<_>>();

    generations.sort_by(|a, b| a.instance.cmp(&b.instance));

    Ok(generations)
}

pub async fn fetch_firing_alerts(
    prometheus_url: String,
    alertmanager_url: String,
    client: reqwest::Client,
) -> Result<Vec<FiringAlert>, String> {
    let prometheus_response = client
        .get(format!("{}/api/v1/query", prometheus_url))
        .query(&[("query", r#"ALERTS{alertstate="firing"}"#)])
        .send()
        .await
        .map_err(|err| format!("failed to query Prometheus alerts: {err}"))?
        .error_for_status()
        .map_err(|err| format!("Prometheus alerts query failed: {err}"))?
        .json::<PrometheusQueryResponse>()
        .await
        .map_err(|err| format!("failed to parse Prometheus alerts response: {err}"))?;

    let alertmanager_alerts = client
        .get(format!(
            "{}/api/v2/alerts",
            alertmanager_url.trim_end_matches('/')
        ))
        .send()
        .await
        .map_err(|err| format!("failed to query Alertmanager alerts: {err}"))?
        .error_for_status()
        .map_err(|err| format!("Alertmanager alerts query failed: {err}"))?
        .json::<Vec<AlertmanagerAlert>>()
        .await
        .map_err(|err| format!("failed to parse Alertmanager alerts response: {err}"))?;

    let alertmanager_silences = client
        .get(format!(
            "{}/api/v2/silences",
            alertmanager_url.trim_end_matches('/')
        ))
        .send()
        .await
        .map_err(|err| format!("failed to query Alertmanager silences: {err}"))?
        .error_for_status()
        .map_err(|err| format!("Alertmanager silences query failed: {err}"))?
        .json::<Vec<AlertmanagerSilence>>()
        .await
        .map_err(|err| format!("failed to parse Alertmanager silences response: {err}"))?;

    let now = Utc::now();

    let active_silence_ends: HashMap<String, DateTime<Utc>> = alertmanager_silences
        .into_iter()
        .filter(|silence| silence.status.state == "active")
        .map(|silence| {
            let ends_at = DateTime::parse_from_rfc3339(&silence.ends_at)
                .map_err(|err| {
                    format!(
                        "failed to parse end time for silence {}: {err}",
                        silence.id
                    )
                })?
                .with_timezone(&Utc);

            Ok((silence.id, ends_at))
        })
        .collect::<Result<HashMap<_, _>, String>>()?;

    let alertmanager_silences_by_key: HashMap<String, Vec<String>> = alertmanager_alerts
        .into_iter()
        .map(|alert| {
            let key = alert_key_from_labels(&alert.labels);
            (key, alert.status.silenced_by)
        })
        .collect();

    let mut alerts = prometheus_response
        .data
        .result
        .into_iter()
        .map(|result| {
            let alertname = result
                .metric
                .get("alertname")
                .cloned()
                .unwrap_or_else(|| "unknown".to_string());

            let name = result
                .metric
                .get("name")
                .cloned()
                .unwrap_or_default();

            let rulegroup = result
                .metric
                .get("rulegroup")
                .cloned()
                .unwrap_or_default();

            let instance = result
                .metric
                .get("instance")
                .cloned()
                .unwrap_or_default();

            let severity = result
                .metric
                .get("severity")
                .cloned()
                .unwrap_or_default();

            let key = format!("{alertname}:{name}:{rulegroup}:{instance}");

            let applicable_silence_ids = alertmanager_silences_by_key
                .get(&key)
                .cloned()
                .unwrap_or_default();

            let remaining_durations = applicable_silence_ids
                .iter()
                .filter_map(|silence_id| active_silence_ends.get(silence_id))
                .map(|ends_at| (*ends_at - now).num_milliseconds() as f64 / 1_000.0)
                .filter(|remaining| *remaining > 0.0)
                .collect::<Vec<_>>();

            let silence_count = remaining_durations.len();

            let longest_silence_remaining_seconds = remaining_durations
                .into_iter()
                .max_by(|a, b| a.total_cmp(b));

            FiringAlert {
                key,
                alertname,
                name,
                rulegroup,
                severity,
                instance,
                timestamp: result.value.0,
                silenced: silence_count > 0,
                silence_count,
                longest_silence_remaining_seconds,
            }
        })
        .collect::<Vec<_>>();

    alerts.sort_by(|a, b| {
        a.severity
            .cmp(&b.severity)
            .then_with(|| a.rulegroup.cmp(&b.rulegroup))
            .then_with(|| a.alertname.cmp(&b.alertname))
            .then_with(|| a.name.cmp(&b.name))
            .then_with(|| a.instance.cmp(&b.instance))
    });

    Ok(alerts)
}

fn alert_key_from_labels(labels: &HashMap<String, String>) -> String {
    let alertname = labels
        .get("alertname")
        .map(String::as_str)
        .unwrap_or("unknown");

    let name = labels.get("name").map(String::as_str).unwrap_or_default();

    let rulegroup = labels
        .get("rulegroup")
        .map(String::as_str)
        .unwrap_or_default();

    let instance = labels
        .get("instance")
        .map(String::as_str)
        .unwrap_or_default();

    format!("{alertname}:{name}:{rulegroup}:{instance}")
}

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

pub async fn fetch_certificate_expiries(
    prometheus_url: String,
    client: reqwest::Client,
) -> Result<Vec<CertificateExpiry>, String> {
    let response = client
        .get(format!("{}/api/v1/query", prometheus_url))
        .query(&[("query", "gatus_results_certificate_expiration_seconds")])
        .send()
        .await
        .map_err(|err| format!("failed to query Prometheus: {err}"))?;

    let prometheus_response = response
        .json::<PrometheusQueryResponse>()
        .await
        .map_err(|err| format!("failed to parse Prometheus response: {err}"))?;

    let mut expiries = prometheus_response
        .data
        .result
        .into_iter()
        .map(|result| CertificateExpiry {
            key: result.metric.get("key").cloned().unwrap_or_default(),
            name: result.metric.get("name").cloned().unwrap_or_default(),
            group: result.metric.get("group").cloned().unwrap_or_default(),
            instance: result.metric.get("instance").cloned().unwrap_or_default(),
            target: result.metric.get("target").cloned().unwrap_or_default(),
            cert_expiry_seconds: result.value.1.parse::<f64>().unwrap_or_default(),
        })
        .collect::<Vec<_>>();

    expiries.sort_by(|a, b| {
        a.cert_expiry_seconds
            .partial_cmp(&b.cert_expiry_seconds)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    Ok(expiries)
}

pub async fn fetch_homelab_tasks(
    prometheus_url: String,
    client: reqwest::Client,
) -> Result<Vec<HomelabTask>, String> {
    let response = client
        .get(format!("{}/api/v1/query", prometheus_url))
        .query(&[(
            "query",
            r#"(time() - homelab_task_last_success_unix) / on(task) group_left homelab_task_allowed_age_seconds"#,
        )])
        .send()
        .await
        .map_err(|err| format!("failed to query Prometheus: {err}"))?;

    let prometheus_response = response
        .json::<PrometheusQueryResponse>()
        .await
        .map_err(|err| format!("failed to parse Prometheus response: {err}"))?;

    let mut tasks = prometheus_response
        .data
        .result
        .into_iter()
        .map(|result| {
            let task = result.metric.get("task").cloned().unwrap_or_default();

            HomelabTask {
                instance: result.metric.get("instance").cloned().unwrap_or_default(),
                name: friendly_name(&task),
                age_ratio: result.value.1.parse::<f64>().unwrap_or_default(),
            }
        })
        .collect::<Vec<_>>();

    tasks.sort_by(|a, b| a.instance.cmp(&b.instance).then_with(|| a.name.cmp(&b.name)));

    Ok(tasks)
}

pub async fn fetch_public_ips(
    prometheus_url: String,
    client: reqwest::Client,
) -> Result<Vec<PublicIpStatus>, String> {
    let response = client
        .get(format!("{}/api/v1/query", prometheus_url))
        .query(&[("query", "homelab_public_ip_numeric")])
        .send()
        .await
        .map_err(|err| format!("failed to query Prometheus: {err}"))?;

    let prometheus_response = response
        .json::<PrometheusQueryResponse>()
        .await
        .map_err(|err| format!("failed to parse Prometheus response: {err}"))?;

    let mut public_ips = prometheus_response
        .data
        .result
        .into_iter()
        .filter_map(|result| {
            let public_ip = result.metric.get("public_ip")?.clone();

            if public_ip.is_empty() {
                return None;
            }

            Some(PublicIpStatus {
                instance: result
                    .metric
                    .get("instance")
                    .cloned()
                    .unwrap_or_default(),
                target: result
                    .metric
                    .get("target")
                    .cloned()
                    .unwrap_or_default(),
                public_ip,
                timestamp: result.value.0,
            })
        })
        .collect::<Vec<_>>();

    public_ips.sort_by(|a, b| a.instance.cmp(&b.instance));

    Ok(public_ips)
}
