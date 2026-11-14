use std::collections::HashMap;

use chrono::{DateTime, Duration as ChronoDuration, Utc};

use super::{
    models::{AlertmanagerAlert, AlertmanagerSilence, CertificateExpiry, DailySloStatus, FiringAlert, GatusHostStatus, HomelabTask, HostUpStatus, LiveCheckStatus, LiveState, NixosGeneration, PrometheusQueryResponse, PrometheusRangeQueryResponse, PublicIpStatus, SloCheckStatus},
    util::{friendly_name, hostname_from_name},
};

const GATUS_SLO_WINDOW_DAYS: i64 = 28;

/// Per-check overrides read from the raw Gatus `slo_*` labels.
#[derive(Default, Clone)]
struct SloPublicCheckMeta {
    slo_name: Option<String>,
    slo_desc: Option<String>,
    slo_url: Option<String>,
}

/// Gatus checks to surface on the public status page: any check whose
/// `slo_public` label is `"true"`, along with their `slo_name`/`slo_desc`/
/// `slo_url` overrides. `homelab_slo_28d`/`homelab_slo_1h` don't carry these
/// labels themselves (they're set on the raw Gatus config, which only
/// `gatus_results_endpoint_success` et al. expose), so this has to be a
/// separate lookup against a raw metric before filtering the recording rules.
async fn fetch_slo_public_checks(
    prometheus_url: &str,
    client: &reqwest::Client,
) -> Result<HashMap<String, SloPublicCheckMeta>, String> {
    let response = client
        .get(format!("{}/api/v1/query", prometheus_url))
        .query(&[("query", r#"gatus_results_endpoint_success{slo_public="true"}"#)])
        .send()
        .await
        .map_err(|err| format!("failed to query Prometheus for slo_public checks: {err}"))?
        .json::<PrometheusQueryResponse>()
        .await
        .map_err(|err| format!("failed to parse Prometheus slo_public checks response: {err}"))?;

    let checks = response
        .data
        .result
        .into_iter()
        .filter_map(|result| {
            let key = result.metric.get("key")?.clone();

            let non_empty = |field: &str| {
                result
                    .metric
                    .get(field)
                    .filter(|value| !value.is_empty())
                    .cloned()
            };

            Some((
                key,
                SloPublicCheckMeta {
                    slo_name: non_empty("slo_name"),
                    slo_desc: non_empty("slo_desc"),
                    slo_url: non_empty("slo_url"),
                },
            ))
        })
        .collect::<HashMap<_, _>>();

    Ok(checks)
}

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
        .query(&[(
            "query",
            r#"gatus_results_endpoint_success{group="Hosts (Local)"} or gatus_results_endpoint_success{group="Hosts (Tailscale)"}"#,
        )])
        .send()
        .await
        .map_err(|err| format!("failed to query Prometheus: {err}"))?;

    let prometheus_response = response
        .json::<PrometheusQueryResponse>()
        .await
        .map_err(|err| format!("failed to parse Prometheus response: {err}"))?;

    let mut hosts: HashMap<String, GatusHostStatus> = HashMap::new();

    for result in prometheus_response.data.result {
        let name = result
            .metric
            .get("name")
            .cloned()
            .unwrap_or_else(|| "unknown".to_string());

        let instance = hostname_from_name(&name);

        let target = result.metric.get("target").cloned().unwrap_or_default();

        hosts.insert(
            instance.clone(),
            GatusHostStatus {
                instance,
                name,
                target,
                timestamp: result.value.0,
                up: result.value.1 == "1",
            },
        );
    }

    let mut statuses = hosts.into_values().collect::<Vec<_>>();

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

pub async fn fetch_gatus_slo(
    prometheus_url: String,
    client: reqwest::Client,
) -> Result<Vec<SloCheckStatus>, String> {
    let slo_public_checks = fetch_slo_public_checks(&prometheus_url, &client).await?;

    if slo_public_checks.is_empty() {
        return Ok(Vec::new());
    }

    let keys = slo_public_checks.keys().cloned().collect::<Vec<_>>();
    let key_regex = gatus_slo_key_regex(&keys);

    let headline_query = format!(r#"homelab_slo_28d{{key=~"{key_regex}"}}"#);

    let headline_response = client
        .get(format!("{}/api/v1/query", prometheus_url))
        .query(&[("query", headline_query.as_str())])
        .send()
        .await
        .map_err(|err| format!("failed to query Prometheus for SLO headline: {err}"))?
        .json::<PrometheusQueryResponse>()
        .await
        .map_err(|err| format!("failed to parse Prometheus SLO headline response: {err}"))?;

    let mut checks: HashMap<String, SloCheckStatus> = HashMap::new();

    for result in headline_response.data.result {
        let key = result.metric.get("key").cloned().unwrap_or_default();
        let meta = slo_public_checks.get(&key).cloned().unwrap_or_default();
        let name = meta
            .slo_name
            .unwrap_or_else(|| result.metric.get("name").cloned().unwrap_or_default());
        let group = result.metric.get("group").cloned().unwrap_or_default();
        let pct_28d = result.value.1.parse::<f64>().unwrap_or_default();

        checks.insert(
            key.clone(),
            SloCheckStatus {
                key,
                name,
                group,
                pct_28d,
                daily: Vec::new(),
                slo_desc: meta.slo_desc,
                slo_url: meta.slo_url,
            },
        );
    }

    let daily_query = format!(
        r#"sum by (key, name, group) (increase(gatus_results_total{{success="true", key=~"{key_regex}"}}[1d])) / sum by (key, name, group) (increase(gatus_results_total{{key=~"{key_regex}"}}[1d]))"#
    );

    let end = Utc::now();
    let start = end - ChronoDuration::days(GATUS_SLO_WINDOW_DAYS);
    let start_param = start.timestamp().to_string();
    let end_param = end.timestamp().to_string();

    let daily_response = client
        .get(format!("{}/api/v1/query_range", prometheus_url))
        .query(&[
            ("query", daily_query.as_str()),
            ("start", start_param.as_str()),
            ("end", end_param.as_str()),
            ("step", "86400"),
        ])
        .send()
        .await
        .map_err(|err| format!("failed to query Prometheus for SLO daily grid: {err}"))?
        .json::<PrometheusRangeQueryResponse>()
        .await
        .map_err(|err| format!("failed to parse Prometheus SLO daily grid response: {err}"))?;

    for result in daily_response.data.result {
        let key = result.metric.get("key").cloned().unwrap_or_default();

        let Some(check) = checks.get_mut(&key) else {
            continue;
        };

        let mut pct_by_date: HashMap<String, f64> = HashMap::new();

        for (timestamp, value) in &result.values {
            if let Ok(pct) = value.parse::<f64>() {
                let date = DateTime::from_timestamp(*timestamp as i64, 0)
                    .map(|dt| dt.format("%Y-%m-%d").to_string())
                    .unwrap_or_default();

                pct_by_date.insert(date, pct);
            }
        }

        check.daily = (0..GATUS_SLO_WINDOW_DAYS)
            .map(|offset| {
                let date = (end - ChronoDuration::days(GATUS_SLO_WINDOW_DAYS - 1 - offset))
                    .format("%Y-%m-%d")
                    .to_string();

                let pct = pct_by_date.get(&date).copied();

                DailySloStatus { date, pct }
            })
            .collect();
    }

    let mut statuses = checks.into_values().collect::<Vec<_>>();
    statuses.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(statuses)
}

/// Live state per check: `Down` if the most recent result failed, `Unstable`
/// if it's currently passing but there was any failure in the last hour
/// (`homelab_slo_1h` < 1.0), otherwise `Healthy`.
pub async fn fetch_gatus_live_status(
    prometheus_url: String,
    client: reqwest::Client,
) -> Result<Vec<LiveCheckStatus>, String> {
    let slo_public_checks = fetch_slo_public_checks(&prometheus_url, &client).await?;

    if slo_public_checks.is_empty() {
        return Ok(Vec::new());
    }

    let keys = slo_public_checks.keys().cloned().collect::<Vec<_>>();
    let key_regex = gatus_slo_key_regex(&keys);

    let current_query = format!(r#"gatus_results_endpoint_success{{key=~"{key_regex}"}}"#);

    let current_response = client
        .get(format!("{}/api/v1/query", prometheus_url))
        .query(&[("query", current_query.as_str())])
        .send()
        .await
        .map_err(|err| format!("failed to query Prometheus for live status: {err}"))?
        .json::<PrometheusQueryResponse>()
        .await
        .map_err(|err| format!("failed to parse Prometheus live status response: {err}"))?;

    let hourly_query = format!(r#"homelab_slo_1h{{key=~"{key_regex}"}}"#);

    let hourly_response = client
        .get(format!("{}/api/v1/query", prometheus_url))
        .query(&[("query", hourly_query.as_str())])
        .send()
        .await
        .map_err(|err| format!("failed to query Prometheus for hourly SLO: {err}"))?
        .json::<PrometheusQueryResponse>()
        .await
        .map_err(|err| format!("failed to parse Prometheus hourly SLO response: {err}"))?;

    let hourly_ratio_by_key: HashMap<String, f64> = hourly_response
        .data
        .result
        .into_iter()
        .filter_map(|result| {
            let key = result.metric.get("key")?.clone();
            let ratio = result.value.1.parse::<f64>().ok()?;
            Some((key, ratio))
        })
        .collect();

    let mut statuses = current_response
        .data
        .result
        .into_iter()
        .map(|result| {
            let key = result.metric.get("key").cloned().unwrap_or_default();
            let currently_up = result.value.1 == "1";

            let state = if !currently_up {
                LiveState::Down
            } else if hourly_ratio_by_key.get(&key).copied().unwrap_or(1.0) < 1.0 {
                LiveState::Unstable
            } else {
                LiveState::Healthy
            };

            LiveCheckStatus { key, state }
        })
        .collect::<Vec<_>>();

    statuses.sort_by(|a, b| a.key.cmp(&b.key));

    Ok(statuses)
}

fn gatus_slo_key_regex(keys: &[String]) -> String {
    escape_promql_string_literal(&format!(
        "^({})$",
        keys.iter()
            .map(|key| escape_regex_literal(key))
            .collect::<Vec<_>>()
            .join("|")
    ))
}

fn escape_regex_literal(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());

    for c in value.chars() {
        if matches!(
            c,
            '\\' | '.' | '^' | '$' | '*' | '+' | '?' | '(' | ')' | '[' | ']' | '{' | '}' | '|'
        ) {
            escaped.push('\\');
        }

        escaped.push(c);
    }

    escaped
}

/// PromQL string literals do their own backslash-escaping before the regex
/// engine ever sees the value, so a single `\(` from `escape_regex_literal`
/// is rejected as an "unknown escape sequence" at the query-parsing stage.
/// Doubling backslashes here makes PromQL unescape `\\(` down to the `\(`
/// the regex engine actually needs to see.
fn escape_promql_string_literal(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}
