#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Traduction des attributs FETCH `X-GM-*`.

use mira_provider_gmail::GmailProvider;
use mira_vocabulary::{Provider, RawMessageAttributes};

/// Construit des attributs bruts à partir de couples
fn attributes(items: &[(&str, &str)]) -> RawMessageAttributes {
    RawMessageAttributes {
        items: items
            .iter()
            .map(|(name, value)| (name.to_string(), value.to_string()))
            .collect(),
    }
}

/// Identifiant stable, fil et libellés sont traduits ensemble
#[test]
fn msgid_thrid_and_labels_are_translated() {
    let raw = attributes(&[
        ("X-GM-MSGID", "1278455344230334865"),
        ("X-GM-THRID", "1278455344230334000"),
        ("X-GM-LABELS", "(\"\\\\Inbox\" \"Travail\" \"Factures\")"),
    ]);
    let translation = GmailProvider.translate_attributes(&raw);
    assert_eq!(
        translation.stable_id.as_deref(),
        Some("1278455344230334865")
    );
    assert_eq!(
        translation.thread_hint.as_deref(),
        Some("1278455344230334000")
    );
    assert_eq!(translation.labels, vec!["\\Inbox", "Travail", "Factures"]);
}

/// Les libellés sans guillemets, avec espaces ou guillemets échappés, sont décodés
#[test]
fn labels_support_atoms_spaces_and_escapes() {
    let raw = attributes(&[(
        "X-GM-LABELS",
        "(\\Sent Important \"Mon dossier\" \"dit \\\"oui\\\"\")",
    )]);
    let translation = GmailProvider.translate_attributes(&raw);
    assert_eq!(
        translation.labels,
        vec!["\\Sent", "Important", "Mon dossier", "dit \"oui\""]
    );
}

/// Liste vide, attributs absents ou inconnus : traduction vide
#[test]
fn empty_and_unknown_attributes_give_an_empty_translation() {
    let empty = attributes(&[("X-GM-LABELS", "()"), ("X-GM-MSGID", "  ")]);
    let translation = GmailProvider.translate_attributes(&empty);
    assert!(translation.labels.is_empty());
    assert_eq!(translation.stable_id, None);
    let unknown = attributes(&[("X-AUTRE", "1")]);
    assert_eq!(
        GmailProvider.translate_attributes(&unknown),
        Default::default()
    );
}

/// Le nom de l'attribut est reconnu quelle que soit sa casse
#[test]
fn attribute_names_are_case_insensitive() {
    let raw = attributes(&[("x-gm-msgid", "42")]);
    let translation = GmailProvider.translate_attributes(&raw);
    assert_eq!(translation.stable_id.as_deref(), Some("42"));
}
