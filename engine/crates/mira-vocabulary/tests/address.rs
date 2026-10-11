#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use mira_vocabulary::{EmailAddress, ErrorCategory};

/// Une adresse ordinaire est acceptée et décomposée en partie locale et domaine.
#[test]
fn valid_address_is_split_into_local_part_and_domain() {
    let address = EmailAddress::parse("alice@example.com").unwrap();
    assert_eq!(address.as_str(), "alice@example.com");
    assert_eq!(address.local_part(), "alice");
    assert_eq!(address.domain(), "example.com");
}

/// Le domaine passe en minuscules, la partie locale reste intacte, les espaces autour tombent.
#[test]
fn domain_is_lowercased_and_surrounding_spaces_are_trimmed() {
    let address = EmailAddress::parse("  Alice.B@Example.COM \t").unwrap();
    assert_eq!(address.as_str(), "Alice.B@example.com");
    assert_eq!(address.local_part(), "Alice.B");
}

/// Table de formes invalides : toutes donnent l'erreur `invalid_input.address`.
#[test]
fn invalid_addresses_are_refused_with_stable_code() {
    let cases = [
        "",
        "alice",
        "@example.com",
        "alice@",
        "a@b@example.com",
        "alice@localhost",
        "alice@example.",
        "alice@.example.com",
        "alice@exa..mple.com",
        "alice@exa mple.com",
        "al ice@example.com",
    ];
    for input in cases {
        let error = EmailAddress::parse(input).unwrap_err();
        assert_eq!(error.category, ErrorCategory::InvalidInput, "{input:?}");
        assert_eq!(error.code, "invalid_input.address", "{input:?}");
    }
}

/// Les adresses internationales en Unicode sont acceptées (12.7).
#[test]
fn unicode_addresses_are_accepted() {
    let address = EmailAddress::parse("josé@Bücher.example").unwrap();
    assert_eq!(address.local_part(), "josé");
    assert_eq!(address.domain(), "bücher.example");
}

/// La désérialisation valide l'adresse comme la saisie et la sérialisation reste une chaîne.
#[test]
fn serde_round_trip_validates_the_address() {
    let address = EmailAddress::parse("a@b.example").unwrap();
    let json = serde_json::to_string(&address).unwrap();
    assert_eq!(json, "\"a@b.example\"");
    assert!(serde_json::from_str::<EmailAddress>("\"pas-une-adresse\"").is_err());
}
