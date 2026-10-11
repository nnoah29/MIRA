#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Dépôt des brouillons et de leurs pièces jointes.

mod support;

use mira_vocabulary::{AccountId, DraftId, DraftState, Timestamp};
use support::accounts::database_with_account;
use support::intentions::{database_with_two_attachments, sample_draft};

/// Un brouillon relu est identique à celui inséré
#[test]
fn inserted_draft_is_read_back_identical() {
    let db = database_with_account("acc-1");
    let draft = sample_draft("d-1", "acc-1", 10);
    db.write(|tx| tx.insert_draft(&draft)).unwrap();
    let loaded = db.read(|tx| tx.get_draft(&draft.id)).unwrap();
    assert_eq!(loaded, Some(draft));
    let unknown = db
        .read(|tx| tx.get_draft(&DraftId::from_raw("zz")))
        .unwrap();
    assert!(unknown.is_none());
}

/// La mise à jour change le contenu et l'état ; un brouillon inconnu est refusé
#[test]
fn update_changes_content_and_unknown_is_refused() {
    let db = database_with_account("acc-1");
    let mut draft = sample_draft("d-1", "acc-1", 10);
    db.write(|tx| tx.insert_draft(&draft)).unwrap();
    draft.subject = "Nouveau sujet".to_string();
    draft.state = DraftState::Scheduled {
        at: Timestamp::from_millis(99),
    };
    db.write(|tx| tx.update_draft(&draft)).unwrap();
    assert_eq!(db.read(|tx| tx.get_draft(&draft.id)).unwrap(), Some(draft));
    let ghost = sample_draft("ghost", "acc-1", 1);
    let error = db.write(|tx| tx.update_draft(&ghost)).unwrap_err();
    assert_eq!(error.code, "not_found.draft");
}

/// La liste d'un compte est triée du plus récent au plus ancien et ignore les autres comptes
#[test]
fn list_is_per_account_and_most_recent_first() {
    let db = database_with_account("acc-1");
    db.write(|tx| {
        tx.insert_draft(&sample_draft("old", "acc-1", 1))?;
        tx.insert_draft(&sample_draft("new", "acc-1", 5))
    })
    .unwrap();
    let drafts = db
        .read(|tx| tx.list_drafts(&AccountId::from_raw("acc-1")))
        .unwrap();
    let ids: Vec<&str> = drafts.iter().map(|draft| draft.id.as_str()).collect();
    assert_eq!(ids, ["new", "old"]);
    let other = db
        .read(|tx| tx.list_drafts(&AccountId::from_raw("acc-2")))
        .unwrap();
    assert!(other.is_empty());
}

/// Les pièces jointes se listent avec leur taille et leur contenu se relit
#[test]
fn attachments_are_listed_with_size_and_readable() {
    let (db, draft) = database_with_two_attachments();
    let listed = db
        .read(|tx| tx.get_draft(&draft.id))
        .unwrap()
        .unwrap()
        .attachments;
    assert_eq!(listed.len(), 2);
    assert_eq!((listed[1].filename.as_str(), listed[1].size), ("b.bin", 4));
    let data = db
        .read(|tx| tx.read_draft_attachment_data(&draft.id, 1))
        .unwrap();
    assert_eq!(data, Some(vec![0, 1, 2, 3]));
}

/// Retirer une pièce jointe ne décale pas les positions des suivantes
#[test]
fn removing_an_attachment_keeps_positions_stable() {
    let (db, draft) = database_with_two_attachments();
    db.write(|tx| tx.remove_draft_attachment(&draft.id, 0))
        .unwrap();
    let after = db.read(|tx| tx.list_draft_attachments(&draft.id)).unwrap();
    assert_eq!(after.len(), 1);
    let next = db
        .write(|tx| tx.add_draft_attachment(&draft.id, "c", "text/plain", b"c"))
        .unwrap();
    assert_eq!(next, 2);
}

/// Supprimer un brouillon supprime ses pièces jointes ; ajouter à un brouillon inconnu est refusé
#[test]
fn delete_cascades_and_attachment_on_unknown_draft_is_refused() {
    let db = database_with_account("acc-1");
    let draft = sample_draft("d-1", "acc-1", 10);
    db.write(|tx| {
        tx.insert_draft(&draft)?;
        tx.add_draft_attachment(&draft.id, "a.txt", "text/plain", b"abc")
    })
    .unwrap();
    db.write(|tx| tx.delete_draft(&draft.id)).unwrap();
    assert_eq!(
        db.read(|tx| tx.table_row_count("draft_attachments"))
            .unwrap(),
        0
    );
    let error = db
        .write(|tx| tx.add_draft_attachment(&draft.id, "x", "text/plain", b"x"))
        .unwrap_err();
    assert_eq!(error.code, "not_found.draft");
    assert!(db.write(|tx| tx.delete_draft(&draft.id)).is_err());
    assert!(
        db.write(|tx| tx.remove_draft_attachment(&draft.id, 0))
            .is_err()
    );
}
