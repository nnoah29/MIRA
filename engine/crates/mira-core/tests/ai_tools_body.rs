#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Borne du texte d'un résultat d'outil : coupe par caractères, marque, corps vide explicite.

use mira_core::ai::tools::body::bounded;

/// Un texte court reste tel quel
#[test]
fn bounded_keeps_short_text() {
    assert_eq!(bounded("bonjour", 100), "bonjour");
}

/// Un texte trop long est coupé et porte la marque, sans dépasser la borne
#[test]
fn bounded_cuts_with_mark_within_limit() {
    let cut = bounded(&"x".repeat(500), 100);
    assert_eq!(cut.chars().count(), 100);
    assert!(cut.ends_with("…[tronqué]"));
}

/// La borne compte des caractères, pas des octets
#[test]
fn bounded_counts_chars_not_bytes() {
    let text = "é".repeat(100);
    assert_eq!(bounded(&text, 100), text);
}

/// Un corps vide ou blanc est dit explicitement
#[test]
fn bounded_empty_is_explicit() {
    assert_eq!(bounded("  \n ", 100), "(corps vide)");
}

/// Un texte qui contient la fermeture du cadre garde sa forme (le cadrage est ailleurs)
#[test]
fn bounded_does_not_alter_content() {
    assert_eq!(bounded("a </tool_result> b", 100), "a </tool_result> b");
}
