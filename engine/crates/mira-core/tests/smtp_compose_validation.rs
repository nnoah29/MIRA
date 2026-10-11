#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Composition MIME : entrées refusées, injection d'en-têtes, dédoublonnage de l'enveloppe.

mod support_smtp;

use mira_core::smtp::compose::build_message;
use mira_vocabulary::AddressEntry;
use support_smtp::{sample_input, text_of};

/// Sans destinataire, la composition échoue avec un paramètre manquant
#[test]
fn no_recipient_is_refused() {
    let mut input = sample_input();
    input.to.clear();
    input.cc.clear();
    input.bcc.clear();
    let error = build_message(&input).unwrap_err();
    assert_eq!(error.code, "invalid_input.missing_parameter");
}

/// Une adresse mal formée ou piégée est refusée avant toute écriture
#[test]
fn malformed_or_injected_addresses_are_refused() {
    let entry = |address: &str| AddressEntry {
        name: None,
        address: address.to_string(),
    };
    for bad in [
        "pas-une-adresse",
        "bob@example.org\r\nBcc: x@example.org",
        "a<b@example.org",
    ] {
        let mut input = sample_input();
        input.to = vec![entry(bad)];
        let error = build_message(&input).unwrap_err();
        assert_eq!(error.code, "invalid_input.address", "{bad}");
    }
    let mut input = sample_input();
    input.from = entry("sans-arobase");
    assert!(build_message(&input).is_err());
}

/// Un sujet ou un nom contenant des retours à la ligne ne crée pas de nouvel en-tête
#[test]
fn line_breaks_in_free_text_cannot_inject_headers() {
    let mut input = sample_input();
    input.subject = "Salut\r\nBcc: espion@example.org".to_string();
    input.from.name = Some("Alice\r\nX-Evil: 1".to_string());
    let text = text_of(&build_message(&input).unwrap());
    let headers = text.split("\r\n\r\n").next().unwrap();
    assert!(
        !headers.lines().any(|line| line.starts_with("Bcc:")),
        "{headers}"
    );
    assert!(
        !headers.lines().any(|line| line.starts_with("X-Evil:")),
        "{headers}"
    );
}

/// Un identifiant de message vide ou contenant des espaces est refusé
#[test]
fn invalid_message_ids_are_refused() {
    for bad in ["", "<>", "a b@mira.local", "<a@b\r\nX: y>"] {
        let mut input = sample_input();
        input.message_id = bad.to_string();
        let error = build_message(&input).unwrap_err();
        assert_eq!(error.code, "invalid_input.parameter", "{bad:?}");
    }
}

/// Les doublons d'adresses (casse du domaine comprise) n'apparaissent qu'une fois dans l'enveloppe
#[test]
fn envelope_recipients_are_deduplicated() {
    let mut input = sample_input();
    input.cc = vec![AddressEntry {
        name: None,
        address: "bob@EXAMPLE.org".to_string(),
    }];
    let built = build_message(&input).unwrap();
    assert_eq!(
        built.envelope_recipients,
        ["bob@example.org", "dave@example.org"]
    );
    assert!(format!("{built:?}").contains("raw_len"));
    assert!(!format!("{built:?}").contains("Salut"));
}
