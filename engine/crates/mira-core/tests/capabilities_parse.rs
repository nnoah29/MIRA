#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Lecture de la réponse CAPABILITY.

use mira_core::capabilities::{ServerCapability, parse_capabilities};

/// Convertit des jetons écrits en texte en liste de chaînes
fn tokens(items: &[&str]) -> Vec<String> {
    items.iter().map(ToString::to_string).collect()
}

/// Les seize extensions connues sont reconnues
#[test]
fn every_known_extension_is_parsed() {
    let parsed = parse_capabilities(&tokens(&[
        "IDLE",
        "NOTIFY",
        "CONDSTORE",
        "QRESYNC",
        "MOVE",
        "UIDPLUS",
        "SPECIAL-USE",
        "OBJECTID",
        "REPLACE",
        "ESEARCH",
        "MULTISEARCH",
        "COMPRESS=DEFLATE",
        "LITERAL+",
        "UTF8=ACCEPT",
        "ID",
        "ENABLE",
    ]));
    assert_eq!(parsed.len(), 16);
    assert!(parsed.contains(&ServerCapability::Compress));
    assert!(parsed.contains(&ServerCapability::Utf8Accept));
    assert!(parsed.contains(&ServerCapability::LiteralPlus));
}

/// La casse est ignorée
#[test]
fn parsing_ignores_case() {
    let parsed = parse_capabilities(&tokens(&["idle", "Qresync", "compress=deflate", "uidplus"]));
    assert!(parsed.contains(&ServerCapability::Idle));
    assert!(parsed.contains(&ServerCapability::Qresync));
    assert!(parsed.contains(&ServerCapability::Compress));
    assert!(parsed.contains(&ServerCapability::UidPlus));
}

/// Les jetons inconnus ou sans intérêt sont ignorés
#[test]
fn unknown_tokens_are_ignored() {
    let parsed = parse_capabilities(&tokens(&["IMAP4rev1", "AUTH=PLAIN", "X-INCONNU", "MOVE"]));
    assert_eq!(parsed.len(), 1);
    assert!(parsed.contains(&ServerCapability::Move));
    assert!(parse_capabilities(&[]).is_empty());
}
