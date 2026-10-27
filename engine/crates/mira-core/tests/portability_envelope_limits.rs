#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Enveloppe d'export YAML : racine invalide, BOM, CRLF, bornes de taille et alias en cascade.

mod support_portability;

use mira_core::portability::envelope::{unwrap, wrap};
use serde_json::{Value, json};
use support_portability::envelope::{code_of, document};

/// Fichier vide ou racine qui n'est pas un objet : erreur typée, pas de panique
#[test]
fn empty_or_non_object_root_is_refused_without_panic() {
    for text in ["", "   \n", "- a\n- b\n", "42\n", "just text\n"] {
        let code = code_of(unwrap(text, "rules", "rules"));
        let typed =
            code == "invalid_input.yaml_syntax" || code == "invalid_input.export_kind_mismatch";
        assert!(typed, "{text:?} → {code}");
    }
}

/// BOM UTF-8 et fins de ligne CRLF (édition sous Windows) sont acceptés
#[test]
fn bom_and_crlf_are_accepted() {
    let text = format!("\u{feff}{}", document("rules", 1, "[]")).replace('\n', "\r\n");
    assert_eq!(
        unwrap(&text, "rules", "rules").unwrap(),
        Vec::<Value>::new()
    );
}

/// Plus de 500 éléments : trop gros
#[test]
fn more_than_500_items_is_too_large() {
    let text = wrap("rules", "rules", vec![json!({"id": "x"}); 501]).unwrap();
    let code = code_of(unwrap(&text, "rules", "rules"));
    assert_eq!(code, "invalid_input.export_too_large");
}

/// Plus de 2 Mo de texte : trop gros, refusé avant toute analyse
#[test]
fn text_over_2_mib_is_too_large() {
    let text = format!(
        "{}# {}\n",
        document("rules", 1, "[]"),
        "x".repeat(2_100_000)
    );
    let code = code_of(unwrap(&text, "rules", "rules"));
    assert_eq!(code, "invalid_input.export_too_large");
}

/// Alias imbriqués (bombe) : le test doit terminer avec une erreur typée
#[test]
fn alias_bomb_is_refused_without_blowup() {
    let head = "mira_export:\n  kind: rules\n  version: 1\nrules:\n  - a0: &a0 [x, x, x, x, x, x, x, x, x]\n";
    let levels: String = (1..40)
        .map(|n| format!("    a{n}: &a{n} [{}]\n", format!("*a{}, ", n - 1).repeat(9)))
        .collect();
    let error = unwrap(&format!("{head}{levels}"), "rules", "rules").unwrap_err();
    assert!(error.code.starts_with("invalid_input."), "{}", error.code);
}

/// Une ancre plate répétée par des alias produit un document énorme : refusé avant toute analyse
#[test]
fn flat_alias_expansion_is_refused() {
    let anchor = format!("[{}]", "x, ".repeat(1500));
    let aliases = "*a, ".repeat(3000);
    let head = "mira_export:\n  kind: rules\n  version: 1\nrules:\n";
    let text = format!("{head}  - a: &a {anchor}\n    b: [{aliases}]\n");
    let code = code_of(unwrap(&text, "rules", "rules"));
    assert_eq!(code, "invalid_input.yaml_syntax");
}
