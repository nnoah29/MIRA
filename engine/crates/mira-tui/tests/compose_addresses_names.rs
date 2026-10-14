//! Noms d'affichage à caractères spéciaux : écrits puis relus à l'identique.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use mira_assembly::mira_vocabulary::AddressEntry;
use mira_tui::state::compose::addresses::{format_addresses, parse_addresses};

/// Adresse de test avec ce nom d'affichage.
fn named(name: &str) -> AddressEntry {
    AddressEntry {
        name: Some(name.to_string()),
        address: "v@x.fr".to_string(),
    }
}

/// Un nom avec `<`, `>`, virgule, guillemets ou accents survit à l'écriture puis à la lecture.
#[test]
fn special_names_round_trip() {
    for name in [
        "Ventes <FR>",
        "a > b",
        "Doe, J",
        "dit \"ok\"",
        "Éloïse Ñandú",
        "<<>>",
        "x\\y",
    ] {
        let entries = vec![named(name), named("Autre")];
        let text = format_addresses(&entries);
        assert_eq!(parse_addresses(&text).unwrap(), entries, "{text}");
    }
}

/// Un nom vide ou absent donne une adresse seule, relue sans nom.
#[test]
fn empty_name_reads_back_as_no_name() {
    let text = format_addresses(&[named("")]);
    assert_eq!(text, "v@x.fr");
    let back = parse_addresses(&text).unwrap();
    assert_eq!(back[0].name, None);
}

/// Une adresse reçue telle quelle d'un courrier, nom entre guillemets avec `<`, n'est pas rejetée.
#[test]
fn quoted_name_with_angle_brackets_is_valid() {
    let parsed = parse_addresses("\"Ventes <FR>\" <v@x.fr>").unwrap();
    assert_eq!(parsed, vec![named("Ventes <FR>")]);
    assert!(parse_addresses("Bob <x> <v@x.fr>").is_err());
}
