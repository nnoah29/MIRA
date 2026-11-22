#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Désabonnement en un clic : jamais vers une machine locale ou privée (audit SEC, constat 2).

use mira_core::mime::parse_message;

/// Message annonçant un désabonnement en un clic vers l'adresse donnée
fn parse(url: &str, mailto: bool) -> Option<mira_vocabulary::Unsubscribe> {
    let extra = if mailto {
        ", <mailto:stop@example.org>"
    } else {
        ""
    };
    let raw = format!(
        "Subject: n\r\nList-Unsubscribe: <{url}>{extra}\r\nList-Unsubscribe-Post: List-Unsubscribe=One-Click\r\n\r\ncorps\r\n"
    );
    parse_message(raw.as_bytes()).unsubscribe
}

/// Les adresses locales, privées et sans domaine sont refusées
#[test]
fn private_and_local_hosts_are_refused() {
    for url in [
        "https://localhost/u",
        "https://127.0.0.1/u",
        "https://10.1.2.3/u",
        "https://172.16.0.1/u",
        "https://172.31.255.1/u",
        "https://192.168.1.10/u",
        "https://169.254.169.254/latest",
        "https://[::1]/u",
        "https://[fd00::1]/u",
        "https://intranet/u",
        "https://0.0.0.0/u",
        "https://printer.local/u",
        "https://user:pw@example.org/u",
    ] {
        assert!(parse(url, false).is_none(), "{url} aurait dû être refusée");
    }
}

/// Une adresse privée seule n'empêche pas de garder le mailto, sans « un clic »
#[test]
fn private_url_keeps_the_mailto_but_not_one_click() {
    let unsubscribe = parse("https://192.168.1.10/u", true).unwrap();
    assert!(unsubscribe.https_url.is_none() && !unsubscribe.one_click);
    assert_eq!(
        unsubscribe.mailto.as_deref(),
        Some("mailto:stop@example.org")
    );
}

/// Les adresses publiques, y compris juste hors des plages privées, sont gardées
#[test]
fn public_hosts_are_kept() {
    for url in [
        "https://example.org/u/1",
        "https://172.32.0.1/u",
        "https://8.8.8.8/u",
        "https://news.example.org:8443/u?id=1",
    ] {
        let unsubscribe = parse(url, false).unwrap();
        assert_eq!(unsubscribe.https_url.as_deref(), Some(url));
        assert!(unsubscribe.one_click);
    }
}
