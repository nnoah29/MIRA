//! Adresses saisies : analyse en `AddressEntry`, écriture, et refus des formes invalides.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use mira_assembly::mira_vocabulary::AddressEntry;
use mira_tui::state::compose::addresses::{format_addresses, parse_addresses};

/// Adresse avec ou sans nom d'affichage.
fn entry(name: Option<&str>, address: &str) -> AddressEntry {
    AddressEntry {
        name: name.map(str::to_string),
        address: address.to_string(),
    }
}

/// Les trois formes courantes sont comprises, séparées par des virgules ou des points-virgules.
#[test]
fn parses_plain_named_and_quoted_forms() {
    let parsed = parse_addresses("a@b.org, Bob <bob@x.org>; \"Doe, John\" <j@d.org>").unwrap();
    let expected = vec![
        entry(None, "a@b.org"),
        entry(Some("Bob"), "bob@x.org"),
        entry(Some("Doe, John"), "j@d.org"),
    ];
    assert_eq!(parsed, expected);
}

/// Écrire puis relire des adresses rend les mêmes adresses, noms à virgule ou guillemets compris.
#[test]
fn format_then_parse_round_trips() {
    let entries = vec![
        entry(Some("Doe, John"), "j@d.org"),
        entry(Some("Dire \"ok\" \\ non"), "q@d.org"),
        entry(Some("日本 語"), "jp@d.org"),
        entry(None, "seule@d.org"),
    ];
    let text = format_addresses(&entries);
    assert_eq!(parse_addresses(&text).unwrap(), entries);
    assert_eq!(format_addresses(&[]), "");
}

/// Un champ vide ou fait de séparateurs ne donne aucune adresse.
#[test]
fn blank_text_has_no_addresses() {
    assert_eq!(parse_addresses("").unwrap(), Vec::new());
    assert_eq!(parse_addresses("  , ; ,").unwrap(), Vec::new());
    assert_eq!(
        parse_addresses("a@b.org,").unwrap(),
        vec![entry(None, "a@b.org")]
    );
}

/// Une adresse sans crochets reste valide avec des espaces autour, et un domaine est gardé tel quel.
#[test]
fn surrounding_spaces_are_trimmed_and_case_is_kept() {
    let parsed = parse_addresses("  <Bob@Example.ORG>  ").unwrap();
    assert_eq!(parsed, vec![entry(None, "Bob@Example.ORG")]);
}
