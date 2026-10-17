//! Brouillons et actions de test.

use super::accounts::database_with_account;
use mira_core::storage::Database;
use mira_vocabulary::{
    AccountId, ActionId, ActionKind, ActionState, AddressEntry, Draft, DraftId, DraftState,
    IdempotencyKey, IdentityId, MailboxId, MessageId, PendingAction, Timestamp,
};

/// Brouillon de test d'un compte, sans pièce jointe
pub fn sample_draft(id: &str, account: &str, updated_at: i64) -> Draft {
    Draft {
        id: DraftId::from_raw(id),
        account: AccountId::from_raw(account),
        identity: IdentityId::from_raw("main"),
        to: vec![AddressEntry {
            name: Some("Bob".to_string()),
            address: "bob@example.org".to_string(),
        }],
        cc: Vec::new(),
        bcc: Vec::new(),
        subject: "Bonjour".to_string(),
        body_text: "Corps du message".to_string(),
        body_html: Some("<p>Corps du message</p>".to_string()),
        attachments: Vec::new(),
        in_reply_to: Some(MessageId::from_raw("msg-9")),
        state: DraftState::Editing,
        updated_at: Timestamp::from_millis(updated_at),
    }
}

/// Base avec un brouillon portant deux pièces jointes (positions 0 et 1)
pub fn database_with_two_attachments() -> (Database, Draft) {
    let db = database_with_account("acc-1");
    let draft = sample_draft("d-1", "acc-1", 10);
    db.write(|tx| {
        tx.insert_draft(&draft)?;
        tx.add_draft_attachment(&draft.id, "a.txt", "text/plain", b"abc")?;
        tx.add_draft_attachment(
            &draft.id,
            "b.bin",
            "application/octet-stream",
            &[0, 1, 2, 3],
        )
    })
    .unwrap();
    (db, draft)
}

/// Action « marquer lu » en attente, avec sa clé d'idempotence
pub fn sample_action(id: &str, account: &str, key: &str, not_before: i64) -> PendingAction {
    PendingAction {
        id: ActionId::from_raw(id),
        account: AccountId::from_raw(account),
        kind: ActionKind::MarkRead { read: true },
        targets: vec![MessageId::from_raw("msg-1")],
        idempotency_key: IdempotencyKey(key.to_string()),
        state: ActionState::Pending,
        attempts: 0,
        last_error: None,
        created_at: Timestamp::from_millis(100),
        not_before: Timestamp::from_millis(not_before),
    }
}

/// Action de déplacement vers une boîte, pour tester les conflits de clé
pub fn sample_move(id: &str, account: &str, key: &str) -> PendingAction {
    let mut action = sample_action(id, account, key, 0);
    action.kind = ActionKind::Move {
        to: MailboxId::from_raw("box-2"),
    };
    action
}
