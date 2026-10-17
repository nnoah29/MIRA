//! Règle AG-03 : le clair (HTTP, SMTP sans TLS) n'est accepté que vers la boucle locale.

use std::net::IpAddr;

use mira_vocabulary::error::codes::tls;
use mira_vocabulary::{ErrorCategory, MiraError};

/// Dit si l'hôte désigne la machine locale (`localhost`, `127.0.0.0/8` ou `::1`)
pub fn is_loopback_host(host: &str) -> bool {
    let bare = host.trim_start_matches('[').trim_end_matches(']');
    bare.eq_ignore_ascii_case("localhost")
        || bare
            .parse::<IpAddr>()
            .is_ok_and(|address| address.is_loopback())
}

/// Refuse une URL qui n'est ni HTTPS ni HTTP vers la boucle locale (`tls.plaintext_refused`)
pub fn require_secure_url(url: &str) -> Result<(), MiraError> {
    let accepted = reqwest::Url::parse(url).is_ok_and(|parsed| match parsed.scheme() {
        "https" => parsed.host_str().is_some(),
        "http" => parsed.host_str().is_some_and(is_loopback_host),
        _ => false,
    });
    if accepted {
        return Ok(());
    }
    Err(MiraError::new(
        ErrorCategory::Tls,
        tls::PLAINTEXT_REFUSED,
        "connexion en clair refusée hors boucle locale",
    ))
}
