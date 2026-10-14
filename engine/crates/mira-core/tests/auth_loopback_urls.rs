#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Règle AG-03 sur les URL : l'hôte est celui qu'un client HTTP contacterait vraiment (revue 15/16).

use mira_core::auth::loopback::{is_loopback_host, require_secure_url};

/// Une barre oblique inverse avant `@` ne déguise pas un hôte distant en boucle locale
#[test]
fn regression_backslash_userinfo_cannot_disguise_a_remote_host() {
    for bad in [
        r"http://evil.example.org\@127.0.0.1/token",
        r"http://evil.example.org\@localhost/token",
        "http://evil.example.org%2f@127.0.0.1.example.org/token",
        "http://127.0.0.1.example.org/token",
    ] {
        assert_eq!(
            require_secure_url(bad).unwrap_err().code,
            "tls.plaintext_refused",
            "{bad}"
        );
    }
}

/// Les formes légitimes de la boucle locale restent acceptées
#[test]
fn loopback_forms_with_userinfo_and_port_are_accepted() {
    for good in [
        "http://user:pw@127.0.0.1:8080/token",
        "http://LOCALHOST:1/x",
        "http://[::1]/x",
        "https://auth.example.org/x",
    ] {
        assert!(require_secure_url(good).is_ok(), "{good}");
    }
}

/// Toute la plage 127.0.0.0/8 est la boucle locale, comme pour les connexions IMAP
#[test]
fn regression_single_loopback_rule_covers_the_whole_127_range() {
    assert!(is_loopback_host("127.0.0.2"));
    assert!(is_loopback_host("[::1]"));
    assert!(is_loopback_host("LocalHost"));
    assert!(!is_loopback_host("10.0.0.1"));
    assert!(!is_loopback_host("127.0.0.1.example.org"));
}
