#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Reconnaissance des adresses du service par domaine exact.

use mira_provider_gmail::GmailProvider;
use mira_vocabulary::{EmailAddress, Provider};

/// Dit si le service gère l'adresse écrite en texte
fn handles(text: &str) -> bool {
    GmailProvider.handles_address(&EmailAddress::parse(text).unwrap())
}

/// Les deux domaines du service sont reconnus
#[test]
fn both_service_domains_are_recognised() {
    assert!(handles("moi@gmail.com"));
    assert!(handles("moi@googlemail.com"));
}

/// Les majuscules du domaine sont normalisées par l'adresse avant la comparaison
#[test]
fn uppercase_domain_is_recognised_after_normalisation() {
    assert!(handles("Moi@GMAIL.COM"));
    assert!(handles("moi@GoogleMail.Com"));
}

/// Un sous-domaine ou un domaine voisin n'est pas reconnu
#[test]
fn subdomains_and_lookalikes_are_refused() {
    for text in [
        "moi@foo.gmail.com",
        "moi@gmail.com.example.org",
        "moi@notgmail.com",
        "moi@gmail.fr",
        "moi@exemple.org",
    ] {
        assert!(!handles(text), "{text}");
    }
}
