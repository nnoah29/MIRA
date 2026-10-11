#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Reconnaissance des adresses du service par domaine exact, régionaux compris.

use mira_provider_outlook::OutlookProvider;
use mira_vocabulary::{EmailAddress, Provider};

/// Dit si le service gère l'adresse écrite en texte
fn handles(text: &str) -> bool {
    OutlookProvider.handles_address(&EmailAddress::parse(text).unwrap())
}

/// Les domaines principaux et régionaux sont reconnus
#[test]
fn main_and_regional_domains_are_recognised() {
    for domain in [
        "outlook.com",
        "hotmail.com",
        "live.com",
        "msn.com",
        "hotmail.fr",
        "outlook.fr",
        "live.fr",
        "hotmail.co.uk",
        "outlook.de",
        "hotmail.it",
        "outlook.es",
        "live.com.au",
        "outlook.com.au",
        "hotmail.ca",
    ] {
        assert!(handles(&format!("moi@{domain}")), "{domain}");
    }
}

/// Les majuscules sont normalisées par l'adresse avant la comparaison
#[test]
fn uppercase_domain_is_recognised_after_normalisation() {
    assert!(handles("Moi@OUTLOOK.COM"));
    assert!(handles("moi@HotMail.Fr"));
}

/// Un sous-domaine ou un domaine voisin n'est pas reconnu
#[test]
fn subdomains_and_lookalikes_are_refused() {
    for text in [
        "moi@foo.outlook.com",
        "moi@outlook.com.example.org",
        "moi@notoutlook.com",
        "moi@outlook.nl",
        "moi@exemple.org",
    ] {
        assert!(!handles(text), "{text}");
    }
}
