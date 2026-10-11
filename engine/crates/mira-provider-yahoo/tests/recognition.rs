#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Reconnaissance des adresses du service par domaine exact, régionaux compris.

use mira_provider_yahoo::YahooProvider;
use mira_vocabulary::{EmailAddress, Provider};

/// Dit si le service gère l'adresse écrite en texte
fn handles(text: &str) -> bool {
    YahooProvider.handles_address(&EmailAddress::parse(text).unwrap())
}

/// Les domaines principaux et régionaux sont reconnus
#[test]
fn main_and_regional_domains_are_recognised() {
    for domain in [
        "yahoo.com",
        "ymail.com",
        "rocketmail.com",
        "yahoo.fr",
        "yahoo.co.uk",
        "yahoo.de",
        "yahoo.it",
        "yahoo.es",
        "yahoo.ca",
        "yahoo.com.au",
        "yahoo.co.jp",
        "yahoo.com.br",
    ] {
        assert!(handles(&format!("moi@{domain}")), "{domain}");
    }
}

/// Les majuscules sont normalisées par l'adresse avant la comparaison
#[test]
fn uppercase_domain_is_recognised_after_normalisation() {
    assert!(handles("Moi@YAHOO.COM"));
    assert!(handles("moi@YMail.Com"));
}

/// Un sous-domaine ou un domaine voisin n'est pas reconnu
#[test]
fn subdomains_and_lookalikes_are_refused() {
    for text in [
        "moi@mail.yahoo.com",
        "moi@yahoo.com.example.org",
        "moi@notyahoo.com",
        "moi@yahoo.nl",
        "moi@exemple.org",
    ] {
        assert!(!handles(text), "{text}");
    }
}
