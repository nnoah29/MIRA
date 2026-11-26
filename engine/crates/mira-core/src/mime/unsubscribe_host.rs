//! Contrôle de l'adresse de désabonnement : une machine locale ou privée n'est jamais jointe.

use std::net::{Ipv4Addr, Ipv6Addr};

use url::{Host, Url};

/// Vrai pour une adresse IPv4 de boucle, privée, locale, partagée ou non routable.
fn is_private_v4(ip: Ipv4Addr) -> bool {
    let [a, b, ..] = ip.octets();
    ip.is_loopback()
        || ip.is_private()
        || ip.is_link_local()
        || ip.is_unspecified()
        || ip.is_broadcast()
        || (a == 100 && (64..128).contains(&b))
}

/// Vrai pour une adresse IPv6 de boucle, locale, unique-locale ou IPv4 embarquée privée.
fn is_private_v6(ip: Ipv6Addr) -> bool {
    let first = ip.segments()[0];
    ip.is_loopback()
        || ip.is_unspecified()
        || first & 0xfe00 == 0xfc00
        || first & 0xffc0 == 0xfe80
        || ip.to_ipv4_mapped().is_some_and(is_private_v4)
}

/// Vrai pour un nom de machine sans domaine public (sans point, `.local`, `.internal`, `.localhost`).
fn is_private_name(name: &str) -> bool {
    let name = name.trim_end_matches('.').to_ascii_lowercase();
    !name.contains('.')
        || ["local", "internal", "localhost", "lan", "home"]
            .iter()
            .any(|suffix| name.ends_with(&format!(".{suffix}")))
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
