#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Enveloppe d'export YAML : aller-retour, version, type et erreurs de syntaxe sans contenu.

mod support_portability;

use mira_core::portability::envelope::{unwrap, wrap};
use serde_json::json;
use support_portability::envelope::{code_of, document};

/// Ce qui est enveloppé ressort identique et dans le même ordre
#[test]
fn wrap_then_unwrap_returns_items_in_order() {
    let items = vec![json!({"id": "b", "n": 2}), json!({"id": "a", "n": 1})];
    let text = wrap("rules", "rules", items.clone()).unwrap();
    assert_eq!(unwrap(&text, "rules", "rules").unwrap(), items);
}

/// Une version inconnue refuse le fichier entier
#[test]
fn unknown_version_refuses_whole_file() {
    let text = document("rules", 2, "[]");
    let code = code_of(unwrap(&text, "rules", "rules"));
    assert_eq!(code, "invalid_input.export_version_unsupported");
}

/// Un autre type de fichier est refusé en entier
#[test]
fn other_kind_refuses_whole_file() {
    let text = document("local_labels", 1, "[]");
    let code = code_of(unwrap(&text, "rules", "rules"));
    assert_eq!(code, "invalid_input.export_kind_mismatch");
}

/// L'erreur de syntaxe donne ligne et colonne, jamais le contenu
#[test]
fn syntax_error_reports_line_and_column_without_content() {
    let error = unwrap("mira_export: [\nSECRET-MARK-42", "rules", "rules").unwrap_err();
    assert_eq!(error.code, "invalid_input.yaml_syntax");
    assert!(error.message.contains("ligne") && error.message.contains("colonne"));
    assert!(!error.message.contains("SECRET-MARK-42"));
}

/// Une ancre YAML est refusée avec un message qui la nomme
#[test]
fn anchor_is_refused_with_explicit_message() {
    let text = document("rules", 1, "[&a x]");
    let error = unwrap(&text, "rules", "rules").unwrap_err();
    assert_eq!(error.code, "invalid_input.yaml_syntax");
    assert!(error.message.contains("ancres"), "{}", error.message);
}

/// Une esperluette au milieu d'un mot ou suivie d'un blanc n'est pas une ancre
#[test]
fn ampersand_inside_text_is_not_an_anchor() {
    let text = document("rules", 1, "[\"R&D\", \"Tom & Jerry\"]");
    assert_eq!(unwrap(&text, "rules", "rules").unwrap().len(), 2);
}
