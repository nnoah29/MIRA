//! Règle AG-03 : le clair (HTTP, SMTP sans TLS) n'est accepté que vers la boucle locale.

use mira_vocabulary::error::codes::tls;
use mira_vocabulary::{ErrorCategory, MiraError};

/// Dit si l'hôte désigne la machine locale (`127.0.0.1`, `::1` ou `localhost`)
pub fn is_loopback_host(host: &str) -> bool {
    let bare = host.trim_start_matches('[').trim_end_matches(']');
    bare == "127.0.0.1" || bare == "::1" || bare.eq_ignore_ascii_case("localhost")
}

/// Refuse une URL qui n'est ni HTTPS ni HTTP vers la boucle locale (`tls.plaintext_refused`)
pub fn require_secure_url(url: &str) -> Result<(), MiraError> {
    if url.starts_with("https://") {
        return Ok(());
    }
    let authority = url.strip_prefix("http://").map(authority_host);
    match authority {
        Some(host) if is_loopback_host(host) => Ok(()),
        _ => Err(MiraError::new(
            ErrorCategory::Tls,
            tls::PLAINTEXT_REFUSED,
            "connexion en clair refusée hors boucle locale",
        )),
    }
}

/// Extrait l'hôte d'un reste d'URL `hôte[:port]/chemin`, crochets IPv6 compris
fn authority_host(rest: &str) -> &str {
    let authority = rest.split(['/', '?', '#']).next().unwrap_or("");
    let authority = authority.rsplit('@').next().unwrap_or(authority);
    if authority.starts_with('[') {
        return authority
            .split_once(']')
            .map_or(authority, |(head, _)| head);
    }
    authority.split(':').next().unwrap_or(authority)
}
