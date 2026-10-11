//! Politique de chiffrement : le clair n'est accepté que vers la boucle locale (AG-03).

use std::net::IpAddr;

use mira_vocabulary::{MiraError, Security, ServerEndpoint};

use crate::imap::errors::tls::plaintext_refused;

/// Indique si l'hôte désigne la boucle locale (`localhost`, `127.0.0.0/8`, `::1`).
pub(crate) fn is_loopback_host(host: &str) -> bool {
    let bare = host.trim_start_matches('[').trim_end_matches(']');
    bare.eq_ignore_ascii_case("localhost")
        || bare
            .parse::<IpAddr>()
            .is_ok_and(|address| address.is_loopback())
}

/// Refuse une connexion en clair vers un hôte qui n'est pas la boucle locale.
pub(crate) fn check_security(endpoint: &ServerEndpoint) -> Result<(), MiraError> {
    if endpoint.security == Security::Plaintext && !is_loopback_host(&endpoint.host) {
        return Err(plaintext_refused(&endpoint.host));
    }
    Ok(())
}
