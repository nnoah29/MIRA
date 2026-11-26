#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Désabonnement en un clic : plages IPv6 réservées refusées (audit SEC, constat 2, mineurs).

use mira_core::mime::parse_message;

/// Vrai si le message garde une adresse https de désabonnement pour cette cible
fn is_kept(url: &str) -> bool {
    let raw = format!(
        "Subject: n\r\nList-Unsubscribe: <{url}>\r\nList-Unsubscribe-Post: List-Unsubscribe=One-Click\r\n\r\ncorps\r\n"
    );
    parse_message(raw.as_bytes()).unsubscribe.is_some()
}

/// Les adresses locales de site obsolètes fec0::/10 sont refusées
#[test]
fn deprecated_site_local_is_refused() {
    assert!(!is_kept("https://[fec0::1]/u") && !is_kept("https://[feff::1]/u"));
}

/// Les adresses IPv4-compatibles ::a.b.c.d sont refusées
#[test]
fn ipv4_compatible_is_refused() {
    assert!(!is_kept("https://[::808:808]/u") && !is_kept("https://[::a00:1]/u"));
}

/// NAT64 vers une IPv4 privée est refusé, vers une IPv4 publique accepté
#[test]
fn nat64_with_private_ipv4_is_refused() {
    assert!(!is_kept("https://[64:ff9b::a00:1]/u") && !is_kept("https://[64:ff9b::7f00:1]/u"));
    assert!(!is_kept("https://[64:ff9b::c0a8:1]/u"));
    assert!(is_kept("https://[64:ff9b::808:808]/u"));
}

/// Le multicast ff00::/8 est refusé
#[test]
fn multicast_is_refused() {
    assert!(!is_kept("https://[ff02::1]/u") && !is_kept("https://[ff0e::1]/u"));
}
