#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Migration v007 et dépôt des clés d'idempotence des règles ; lectures utiles aux règles (candidats, fil existant).

mod support;
mod support_mirror;

use mira_core::storage::Database;
use mira_core::storage::repositories::rules::RuleIdempotencyEntry;
use mira_vocabulary::{AccountId, IdempotencyKey, MailboxRole, MessageId, RuleId, RuleScope};
use mira_vocabulary::{ThreadId, Timestamp};
use support::accounts::database_with_account;
use support::rules::sample_rule;
use support_mirror::fixtures::{appearance, sample_mailbox, sample_message};

/// Une clé enregistrée se relit à l'identique, survit à la suppression de sa règle ; une clé inconnue ne rend rien
#[test]
fn recorded_key_survives_rule_deletion() {
    let db = database_with_account("acc-1");
    let rule = sample_rule("r-1", RuleScope::AllAccounts);
    let key = IdempotencyKey("k-1".to_string());
    let entry = RuleIdempotencyEntry {
        operation: "delete_rule".to_string(),
        fingerprint: "\"r-1\"".to_string(),
        rule: Some(RuleId::from_raw("r-1")),
    };
    db.write(|tx| {
        tx.insert_rule(&rule)?;
        tx.delete_rule(&rule.id)?;
        tx.record_rule_idempotency(&key, &entry)
    })
    .unwrap();
    let found = db.read(|tx| tx.rule_idempotency_entry(&key)).unwrap();
    assert_eq!(found, Some(entry.clone()));
    let unknown = IdempotencyKey("k-2".to_string());
    assert_eq!(
        db.read(|tx| tx.rule_idempotency_entry(&unknown)).unwrap(),
        None
    );
    let again = db.write(|tx| tx.record_rule_idempotency(&key, &entry));
    assert!(again.is_err());
}

/// Une boîte par rôle avec un message chacune, plus « m-late », le plus récent, en réception et en Corbeille
fn database_with_one_message_per_role() -> Database {
    let db = database_with_account("acc-1");
    let boxes = [
        ("in", MailboxRole::Inbox),
        ("trash", MailboxRole::Trash),
        ("sent", MailboxRole::Sent),
        ("drafts", MailboxRole::Drafts),
        ("junk", MailboxRole::Junk),
    ];
    db.write(|tx| {
        for (rank, (name, role)) in boxes.into_iter().enumerate() {
            tx.upsert_mailbox(&sample_mailbox(name, "acc-1", name, role))?;
            let id = format!("m-{name}");
            tx.insert_message(
                &sample_message(&id, "acc-1"),
                Timestamp::from_millis(rank as i64),
            )?;
            tx.add_appearance(&appearance(&id, name, 1))?;
        }
        tx.insert_message(
            &sample_message("m-late", "acc-1"),
            Timestamp::from_millis(99),
        )?;
        tx.add_appearance(&appearance("m-late", "in", 2))?;
        tx.add_appearance(&appearance("m-late", "trash", 2))
    })
    .unwrap();
    db
}

/// Les candidats excluent les messages présents seulement en Corbeille, Envoyés, Brouillons ou Spam, et suivent la date
#[test]
fn candidates_skip_excluded_mailboxes_and_follow_date() {
    let db = database_with_one_message_per_role();
    let found = db
        .read(|tx| tx.rule_candidates(&AccountId::from_raw("acc-1"), 10))
        .unwrap();
    let ids: Vec<&str> = found.iter().map(|(_, m)| m.id.as_str()).collect();
    assert_eq!(ids, ["m-late", "m-in"]);
    let limited = db
        .read(|tx| tx.rule_candidates(&AccountId::from_raw("acc-1"), 1))
        .unwrap();
    assert_eq!(limited.len(), 1);
}

/// Un fil « existe déjà » pour un message s'il contient un autre message que lui
#[test]
fn thread_has_other_messages_ignores_the_message_itself() {
    let db = database_with_account("acc-1");
    let thread = ThreadId::from_raw("t-1");
    db.write(|tx| {
        tx.upsert_thread(&AccountId::from_raw("acc-1"), "<root>", "Sujet", &thread)?;
        for id in ["m-1", "m-2"] {
            tx.insert_message(&sample_message(id, "acc-1"), Timestamp::from_millis(1))?;
        }
        tx.set_thread(&MessageId::from_raw("m-1"), Some(&thread))
    })
    .unwrap();
    let alone = db.read(|tx| tx.thread_has_other_messages(&thread, &MessageId::from_raw("m-1")));
    assert!(!alone.unwrap());
    db.write(|tx| tx.set_thread(&MessageId::from_raw("m-2"), Some(&thread)))
        .unwrap();
    let other = db.read(|tx| tx.thread_has_other_messages(&thread, &MessageId::from_raw("m-1")));
    assert!(other.unwrap());
}
