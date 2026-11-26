//! Contrôle de l'adresse de désabonnement : une machine locale ou privée n'est jamais jointe.

use url::{Host, Url};

use super::unsubscribe_ranges::{is_private_v4, is_private_v6};

/// Vrai pour un nom de machine sans domaine public (sans point, `.local`, `.internal`, `.localhost`).
fn is_private_name(name: &str) -> bool {
    let name = name.trim_end_matches('.').to_ascii_lowercase();
    !name.contains('.')
        || ["local", "internal", "localhost", "lan", "home"]
            .iter()
            .any(|suffix| {
                name.strip_suffix(suffix)
                    .is_some_and(|rest| rest.ends_with('.'))
            })
}

/// Vrai si l'hôte est une machine locale, privée ou réservée (adresse ou nom sans domaine public).
pub(crate) fn is_private_host(host: Host<&str>) -> bool {
    match host {
        Host::Domain(name) => is_private_name(name),
        Host::Ipv4(ip) => is_private_v4(ip),
        Host::Ipv6(ip) => is_private_v6(ip),
    }
}

/// Vrai si l'adresse est en https, sans identifiants, et vise une machine publique.
pub(crate) fn is_public_https(url: &str) -> bool {
    let Ok(parsed) = Url::parse(url) else {
        return false;
    };
    if parsed.scheme() != "https" || !parsed.username().is_empty() || parsed.password().is_some() {
        return false;
    }
    match parsed.host() {
        Some(Host::Domain(name)) => !is_private_name(name),
        Some(Host::Ipv4(ip)) => !is_private_v4(ip),
        Some(Host::Ipv6(ip)) => !is_private_v6(ip),
        None => false,
    }
}
