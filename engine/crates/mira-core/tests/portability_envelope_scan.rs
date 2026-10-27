#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Contrôle des alias et des clés en double avant lecture, sans faux positif sur le texte des noms.

mod support_portability;

use mira_core::portability::envelope::{unwrap, wrap};
use serde_json::{Value, json};
use support_portability::envelope::document;

/// Esperluettes et étoiles dans des noms écrits par l'export lui-même
fn tricky_names() -> Vec<&'static str> {
    vec![
        "Moi &Toi",
        "Ventes: &Achats",
        "a,&b",
        "x -&y",
        "&début",
        "*étoile",
        "- tiret",
        "# dièse",
        "oui",
        "null",
        "~",
        "a: b",
        "\"guillemets\" et 'apostrophes'",
        "é ü 日本語",
    ]
}

/// Un alias est refusé, avec sa position
#[test]
fn alias_is_refused_with_position() {
    let text = document("rules", 1, "[&a x, *a]");
    let error = unwrap(&text, "rules", "rules").unwrap_err();
    assert_eq!(error.code, "invalid_input.yaml_syntax");
    assert!(error.message.contains("alias"), "{}", error.message);
    assert!(error.message.contains("ligne") && error.message.contains("colonne"));
}

/// Une ancre que personne ne reprend ne fait aucun mal
#[test]
fn anchor_without_alias_is_accepted() {
    let text = document("rules", 1, "[&a x, y]");
    assert_eq!(unwrap(&text, "rules", "rules").unwrap().len(), 2);
}

/// Régression : les noms écrits par l'export se relisent tous, quelle que soit leur ponctuation
#[test]
fn names_with_ampersands_survive_a_round_trip() {
    let items: Vec<Value> = tricky_names()
        .into_iter()
        .map(|n| json!({ "name": n }))
        .collect();
    let text = wrap("rules", "rules", items.clone()).unwrap();
    assert_eq!(unwrap(&text, "rules", "rules").unwrap(), items);
}

/// Deux fichiers collés bout à bout : la clé `rules` en double est refusée avec sa position
#[test]
fn concatenated_files_are_refused_not_half_read() {
    let text = format!(
        "{}{}",
        document("rules", 1, "[]"),
        document("rules", 1, "[]")
    );
    let error = unwrap(&text, "rules", "rules").unwrap_err();
    assert_eq!(error.code, "invalid_input.yaml_syntax");
    assert!(error.message.contains("double"), "{}", error.message);
    assert!(error.message.contains("ligne"), "{}", error.message);
}

/// Une clé répétée dans une entrée est refusée sans citer la clé
#[test]
fn duplicate_key_inside_an_item_is_refused_without_naming_it() {
    let text = document("rules", 1, "[{name: A, SECRET-KEY: 1, SECRET-KEY: 2}]");
    let error = unwrap(&text, "rules", "rules").unwrap_err();
    assert!(error.message.contains("double"), "{}", error.message);
    assert!(!error.message.contains("SECRET-KEY"), "{}", error.message);
}
