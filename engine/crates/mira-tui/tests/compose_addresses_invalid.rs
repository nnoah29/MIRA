//! Adresses saisies : formes invalides, signalées telles qu'elles ont été saisies.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use mira_tui::state::compose::addresses::parse_addresses;

/// Une adresse invalide est rendue telle qu'elle a été saisie, pour être signalée.
#[test]
fn invalid_address_is_reported_with_its_text() {
    assert_eq!(
        parse_addresses("a@b.org, pas une adresse"),
        Err("pas une adresse".to_string())
    );
    assert_eq!(
        parse_addresses("sans-arobase"),
        Err("sans-arobase".to_string())
    );
    assert_eq!(parse_addresses("x@y"), Err("x@y".to_string()));
}

/// Les crochets mal fermés ou suivis de texte, et les guillemets ouverts, sont invalides.
#[test]
fn malformed_brackets_and_quotes_are_invalid() {
    for bad in [
        "Bob <bob@x.org> reste",
        "Bob <bob@x.org",
        "Bob bob@x.org>",
        "\"Bob <bob@x.org>",
        "<>",
        "Bob <>",
    ] {
        assert!(parse_addresses(bad).is_err(), "{bad}");
    }
}
