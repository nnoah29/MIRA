#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Apparitions provisoires (message déplacé hors ligne), renommage de boîte et charge utile des actions.

mod support;
mod support_mirror;

use mira_core::storage::repositories::actions::ActionPayload;
use mira_vocabulary::{AccountId, ActionId, MailboxId, MessageId};
use support::intentions::sample_action;
use support_mirror::fixtures::{appearance, sample_mailbox};
use support_mirror::seeded::database_with_messages;

/// Une apparition provisoire n'a pas d'UID serveur, est à revalider et n'entre pas dans les UID de la boîte
#[test]
fn local_appearance_has_no_server_uid_and_is_not_listed_by_uid() {
    let db = database_with_messages(2);
    let other = MailboxId::from_raw("box-3");
    db.write(|tx| {
        tx.upsert_mailbox(&sample_mailbox(
            "box-3",
            "acc-1",
            "Autre",
            mira_vocabulary::MailboxRole::Other,
        ))?;
        tx.add_local_appearance(&MessageId::from_raw("msg-1"), &other, false)?;
        tx.add_local_appearance(&MessageId::from_raw("msg-2"), &other, false)
    })
    .unwrap();
    let found = db
        .read(|tx| tx.get_appearance(&MessageId::from_raw("msg-1"), &other))
        .unwrap()
        .unwrap();
    assert_eq!(found.uid, 0);
    assert!(found.needs_revalidation);
    assert!(db.read(|tx| tx.uids_in_mailbox(&other)).unwrap().is_empty());
    let all = db.read(|tx| tx.appearances_in_mailbox(&other)).unwrap();
    assert_eq!(all.len(), 2);
}

/// Un ajout provisoire est sans effet si le message est déjà dans la boîte
#[test]
fn local_appearance_does_not_replace_a_real_one() {
    let db = database_with_messages(1);
    let message = MessageId::from_raw("msg-1");
    let inbox = MailboxId::from_raw("box-1");
    db.write(|tx| tx.add_local_appearance(&message, &inbox, false))
        .unwrap();
    let kept = db
        .read(|tx| tx.get_appearance(&message, &inbox))
        .unwrap()
        .unwrap();
    assert_eq!(kept.uid, 1);
    assert!(!kept.needs_revalidation);
}

/// La synchro qui voit enfin le message sous un UID remplace l'apparition provisoire
#[test]
fn real_appearance_replaces_the_local_placeholder() {
    let db = database_with_messages(1);
    let message = MessageId::from_raw("msg-1");
    let other = MailboxId::from_raw("box-3");
    db.write(|tx| {
        tx.upsert_mailbox(&sample_mailbox(
            "box-3",
            "acc-1",
            "Autre",
            mira_vocabulary::MailboxRole::Other,
        ))?;
        tx.add_local_appearance(&message, &other, false)?;
        tx.add_appearance(&appearance("msg-1", "box-3", 12))
    })
    .unwrap();
    let real = db
        .read(|tx| tx.get_appearance(&message, &other))
        .unwrap()
        .unwrap();
    assert_eq!(real.uid, 12);
    assert!(!real.needs_revalidation);
    let again = db.write(|tx| tx.add_appearance(&appearance("msg-1", "box-3", 13)));
    assert!(again.is_err(), "un doublon réel reste refusé");
}

/// Renommer une boîte change le préfixe de ses descendantes et rend l'ancien nom
#[test]
fn rename_mailbox_rewrites_descendant_names() {
    let db = database_with_messages(1);
    let account = AccountId::from_raw("acc-1");
    let old = db
        .write(|tx| {
            for (id, name) in [("p", "A"), ("c", "A/B"), ("x", "AB")] {
                let role = mira_vocabulary::MailboxRole::Other;
                tx.upsert_mailbox(&sample_mailbox(id, "acc-1", name, role))?;
            }
            tx.rename_mailbox(&MailboxId::from_raw("p"), "Z", "Z")
        })
        .unwrap();
    assert_eq!(old, "A");
    let names: Vec<String> = db
        .read(|tx| tx.list_mailboxes(&account))
        .unwrap()
        .into_iter()
        .map(|m| m.name)
        .collect();
    assert!(names.contains(&"Z".to_string()));
    assert!(names.contains(&"Z/B".to_string()));
    assert!(
        names.contains(&"AB".to_string()),
        "un nom voisin n'est pas touché"
    );
}

/// La charge utile d'une action (cibles serveur, inverse) se relit telle qu'écrite
#[test]
fn action_payload_round_trips_and_requeue_resets_the_failure() {
    let db = support::accounts::database_with_account("acc-1");
    let action = sample_action("a-1", "acc-1", "key", 0);
    let payload = ActionPayload {
        server_targets_json: "{\"messages\":[],\"mailboxes\":[]}".to_string(),
        undo_json: Some("{}".to_string()),
    };
    db.write(|tx| {
        tx.enqueue_action(&action)?;
        tx.record_action_failure(&action.id, "x", mira_vocabulary::Timestamp::from_millis(9))?;
        tx.set_action_payload(&action.id, &payload)
    })
    .unwrap();
    let id: &ActionId = &action.id;
    assert_eq!(
        db.read(|tx| tx.get_action_payload(id)).unwrap(),
        Some(payload)
    );
    db.write(|tx| tx.requeue_action(id, mira_vocabulary::Timestamp::from_millis(5)))
        .unwrap();
    let stored = db.read(|tx| tx.get_action(id)).unwrap().unwrap();
    assert_eq!((stored.attempts, stored.last_error), (0, None));
    assert_eq!(stored.not_before.as_millis(), 5);
}
