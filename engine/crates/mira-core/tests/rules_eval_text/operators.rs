//! Opérateurs de comparaison de texte.

use mira_core::rules::text_match::matches_text;
use mira_vocabulary::{TextMatcher, TextOp};

/// Filtre à une liste de valeurs
fn matcher(op: TextOp, values: &[&str]) -> TextMatcher {
    TextMatcher {
        op,
        values: values.iter().map(|v| v.to_string()).collect(),
    }
}

/// La casse et les accents majuscules sont ignorés pour tous les opérateurs de texte
#[test]
fn contains_ignores_case_and_accents_case() {
    assert!(matches_text(
        &matcher(TextOp::Contains, &["été"]),
        "Facture ÉTÉ 2026"
    ));
    assert!(matches_text(&matcher(TextOp::Equals, &["É"]), "é"));
    assert!(matches_text(
        &matcher(TextOp::StartsWith, &["éco"]),
        "ÉCOLE"
    ));
    assert!(matches_text(
        &matcher(TextOp::EndsWith, &["ÉTÉ"]),
        "bel été"
    ));
    assert!(!matches_text(&matcher(TextOp::Equals, &["e"]), "é"));
}

/// Une regex se compare sans tenir compte de la casse
#[test]
fn regex_matches() {
    assert!(matches_text(
        &matcher(TextOp::Regex, &[r"^fact\w+ \d{4}$"]),
        "Facture 2026"
    ));
    assert!(!matches_text(&matcher(TextOp::Regex, &[r"^\d+$"]), "abc"));
}

/// Une regex invalide est fausse, sans panique
#[test]
fn invalid_regex_is_false() {
    assert!(!matches_text(&matcher(TextOp::Regex, &["("]), "("));
    assert!(!matches_text(
        &matcher(TextOp::Regex, &["a{1000000}{1000000}"]),
        "a"
    ));
}

/// `in_list` et les autres opérateurs acceptent une valeur parmi plusieurs
#[test]
fn in_list_matches_any() {
    let list = matcher(TextOp::InList, &["Alpha", "bêta"]);
    assert!(matches_text(&list, "ALPHA"));
    assert!(matches_text(&list, "BÊTA"));
    assert!(!matches_text(&list, "gamma"));
    assert!(!matches_text(&matcher(TextOp::InList, &[]), "alpha"));
}
