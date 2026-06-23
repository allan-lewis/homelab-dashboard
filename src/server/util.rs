use crate::server::models::HostUpStatus;

pub fn hostname_from_name(name: &str) -> String {
    name.split_whitespace()
        .next()
        .unwrap_or("unknown")
        .to_lowercase()
}

pub fn persona_from_name(name: &str) -> String {
    let Some(start) = name.find('(') else {
        return String::new();
    };

    let Some(end) = name[start + 1..].find(')') else {
        return String::new();
    };

    name[start + 1..start + 1 + end].to_string()
}

pub fn ip_address_for_host(hostname: &str, up_statuses: &[HostUpStatus]) -> String {
    up_statuses
        .iter()
        .find(|status| status.instance == hostname)
        .and_then(|status| status.target.split_once(':').map(|(ip, _)| ip.to_string()))
        .unwrap_or_default()
}

pub fn friendly_name(value: &str) -> String {
    value
        .split('_')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();

            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}
