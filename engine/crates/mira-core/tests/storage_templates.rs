#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Dépôt des modèles de messages : tri, nom unique, bornes.

mod support_templates;

use mira_core::storage::Database;
use support_templates::{code_of, insert, template};

/// Les modèles sont rendus par nom sans tenir compte de la casse
#[test]
fn insert_then_list_sorted_case_insensitive() {
    let db = Database::open_in_memory().unwrap();
    insert(&db, &template("t-1", "zèbre")).unwrap();
    insert(&db, &template("t-2", "Alpha")).unwrap();
    let names: Vec<String> = db
        .read(|tx| tx.list_templates())
        .unwrap()
        .into_iter()
        .map(|item| item.name)
        .collect();
    assert_eq!(names, ["Alpha", "zèbre"]);
}

/// Un nom qui ne diffère que par la casse ou les espaces de bord est un doublon
#[test]
fn name_is_unique_ignoring_case_and_trim() {
    let db = Database::open_in_memory().unwrap();
    insert(&db, &template("t-1", "Merci")).unwrap();
    let again = insert(&db, &template("t-2", "  merci "));
    assert_eq!(code_of(again), "invalid_input.template_name_taken");
    assert_eq!(db.read(|tx| tx.count_templates()).unwrap(), 1);
}

/// Un nom vide ou d'espaces est refusé
#[test]
fn empty_name_is_refused() {
    let db = Database::open_in_memory().unwrap();
    let refused = insert(&db, &template("t-1", "   "));
    assert_eq!(code_of(refused), "invalid_input.template_name_empty");
}

/// Un nom, un objet ou un corps au-delà des bornes est refusé sans rien écrire
#[test]
fn too_long_fields_are_refused() {
    let db = Database::open_in_memory().unwrap();
    let long_name = template("t-1", &"n".repeat(101));
    let mut long_subject = template("t-2", "Objet long");
    long_subject.subject = Some("s".repeat(301));
    let mut long_body = template("t-3", "Corps long");
    long_body.body = "b".repeat(100_001);
    for item in [long_name, long_subject, long_body] {
        assert_eq!(
            code_of(insert(&db, &item)),
            "invalid_input.template_too_long"
        );
    }
    assert_eq!(db.read(|tx| tx.count_templates()).unwrap(), 0);
}

/// Les bornes sont atteignables : exactement 100, 300 et 100 000 caractères passent
#[test]
fn fields_at_the_limit_are_accepted() {
    let db = Database::open_in_memory().unwrap();
    let mut item = template("t-1", &"é".repeat(100));
    item.subject = Some("s".repeat(300));
    item.body = "b".repeat(100_000);
    insert(&db, &item).unwrap();
}
