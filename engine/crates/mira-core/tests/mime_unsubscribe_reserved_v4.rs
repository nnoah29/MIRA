#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Désabonnement en un clic : plages IPv4 réservées refusées (audit SEC, constat 2, mineurs).

use mira_core::mime::parse_message;

/// Vrai si le message garde une adresse https de désabonnement pour cette cible
fn is_kept(url: &str) -> bool {
    let raw = format!(
        "Subject: n\r\nList-Unsubscribe: <{url}>\r\nList-Unsubscribe-Post: List-Unsubscribe=One-Click\r\n\r\ncorps\r\n"
    );
    parse_message(raw.as_bytes()).unsubscribe.is_some()
}

/// La diffusion limitée est refusée
#[test]
fn limited_broadcast_is_refused() {
    assert!(!is_kept("https://255.255.255.255/u"));
}

/// Les adresses réservées 240.0.0.0/4 sont refusées, la plage publique voisine reste acceptée
#[test]
fn reserved_class_e_is_refused() {
    assert!(!is_kept("https://240.0.0.1/u") && !is_kept("https://250.1.2.3/u"));
    assert!(is_kept("https://223.255.255.1/u"));
}

/// Les adresses d'évaluation de performance 198.18.0.0/15 sont refusées
#[test]
fn benchmark_range_is_refused() {
    assert!(!is_kept("https://198.18.0.1/u") && !is_kept("https://198.19.255.254/u"));
    assert!(is_kept("https://198.17.255.1/u") && is_kept("https://198.20.0.1/u"));
}

/// Les adresses du protocole IETF 192.0.0.0/24 sont refusées
#[test]
fn ietf_protocol_range_is_refused() {
    assert!(!is_kept("https://192.0.0.8/u") && !is_kept("https://192.0.0.170/u"));
    assert!(is_kept("https://192.0.1.1/u"));
}
