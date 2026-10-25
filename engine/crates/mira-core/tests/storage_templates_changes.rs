#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Dépôt des modèles de messages : mise à jour, suppression, migration.

mod support_templates;

use mira_core::storage::{Database, schema_version};
use mira_vocabulary::TemplateId;
use support_templates::{code_of, insert, template};

/// La mise à jour garde l'identifiant et la création, change les champs et la date de mise à jour
#[test]
fn update_keeps_id_and_changes_updated_at() {
    let db = Database::open_in_memory().unwrap();
    insert(&db, &template("t-1", "Merci")).unwrap();
    let id = TemplateId::from_raw("t-1");
    db.write(|tx| tx.update_template(&id, "Remerciement", None, "Corps", 9))
        .unwrap();
    let found = db.read(|tx| tx.get_template(&id)).unwrap().unwrap();
    assert_eq!(found.name, "Remerciement");
    assert_eq!(found.subject, None);
    assert_eq!((found.created_at, found.updated_at), (1, 9));
}

/// Renommer vers le nom d'un autre modèle est refusé ; garder son propre nom est permis
#[test]
fn update_to_another_templates_name_is_refused() {
    let db = Database::open_in_memory().unwrap();
    insert(&db, &template("t-1", "Merci")).unwrap();
    insert(&db, &template("t-2", "Relance")).unwrap();
    let id = TemplateId::from_raw("t-2");
    let taken = db.write(|tx| tx.update_template(&id, "merci", None, "x", 2));
    assert_eq!(code_of(taken), "invalid_input.template_name_taken");
    db.write(|tx| tx.update_template(&id, "RELANCE", None, "x", 2))
        .unwrap();
}

/// Supprimer ou modifier un modèle inconnu donne `not_found.template`
#[test]
fn unknown_template_is_not_found() {
    let db = Database::open_in_memory().unwrap();
    let id = TemplateId::from_raw("absent");
    let deleted = db.write(|tx| tx.delete_template(&id));
    assert_eq!(code_of(deleted), "not_found.template");
    let updated = db.write(|tx| tx.update_template(&id, "Nom", None, "x", 1));
    assert_eq!(code_of(updated), "not_found.template");
}

/// La table existe au dernier schéma et les modèles survivent à la réouverture
#[test]
fn migration_creates_table_and_data_survives_reopen() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("mira.sqlite3");
    let db = Database::open(&path).unwrap();
    assert!(schema_version() >= 12);
    assert!(
        db.read(|tx| tx.table_names())
            .unwrap()
            .contains(&"message_templates".to_string())
    );
    insert(&db, &template("t-1", "Merci")).unwrap();
    drop(db);
    let reopened = Database::open(&path).unwrap();
    assert_eq!(reopened.read(|tx| tx.count_templates()).unwrap(), 1);
}
