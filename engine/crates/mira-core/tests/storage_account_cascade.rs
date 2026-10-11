#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Suppression d'un compte : tout ce qui en dépend disparaît, les événements restent.

mod support;

use mira_vocabulary::{AccountId, DraftId, EventKind, Timestamp};
use support::accounts::{database_with_account, sample_account, sample_identity};
use support::intentions::{sample_action, sample_draft};

/// Supprimer un compte retire ses identités, brouillons et actions sans toucher l'autre compte
#[test]
fn deleting_an_account_removes_dependents_only() {
    let db = database_with_account("acc-1");
    db.write(|tx| {
        tx.insert_account(&sample_account("acc-2"))?;
        tx.insert_identity(&sample_identity("main", "acc-1", true))?;
        tx.insert_draft(&sample_draft("d-1", "acc-1", 1))?;
        tx.add_draft_attachment(&DraftId::from_raw("d-1"), "a", "text/plain", b"a")?;
        tx.enqueue_action(&sample_action("a-1", "acc-1", "k1", 0))?;
        tx.insert_draft(&sample_draft("d-2", "acc-2", 1))?;
        tx.append_event(Timestamp::from_millis(1), &EventKind::EngineStarted)?;
        tx.delete_account(&AccountId::from_raw("acc-1"))
    })
    .unwrap();
    let count = |table: &str| db.read(|tx| tx.table_row_count(table)).unwrap();
    assert_eq!(count("accounts"), 1);
    assert_eq!(count("identities"), 0);
    assert_eq!(count("drafts"), 1);
    assert_eq!(count("draft_attachments"), 0);
    assert_eq!(count("actions"), 0);
    assert_eq!(count("events"), 1);
}
