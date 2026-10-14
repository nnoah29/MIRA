//! Miroir garni : trois boîtes, cinq messages dans la réception, deux fils, deux corps téléchargés.

use mira_assembly::mira_vocabulary::{
    AccountId, Appearance, ContentState, MailboxId, MailboxRole, MessageId, MiraError, ThreadId,
    Timestamp,
};
use mira_assembly::storage::{Database, Tx};

use super::seed_fixtures::{mailbox, message, message_rows};

/// Crée INBOX (`box-in`), Archive (`box-arch`), Trash (`box-trash`) et Sent (`box-sent`).
fn seed_mailboxes(tx: &Tx, account: &str) -> Result<(), MiraError> {
    tx.upsert_mailbox(&mailbox("box-in", account, "INBOX", MailboxRole::Inbox))?;
    tx.upsert_mailbox(&mailbox(
        "box-arch",
        account,
        "Archive",
        MailboxRole::Archive,
    ))?;
    tx.upsert_mailbox(&mailbox("box-trash", account, "Trash", MailboxRole::Trash))?;
    tx.upsert_mailbox(&mailbox("box-sent", account, "Sent", MailboxRole::Sent))?;
    Ok(())
}

/// Range m1 à m5 dans INBOX sous les UID 1 à 5, datés de 1 s à 5 s.
fn seed_messages(tx: &Tx, account: &str) -> Result<(), MiraError> {
    for (index, (id, sender, flags)) in message_rows().into_iter().enumerate() {
        let sorted_at = Timestamp::from_millis(1_000 * (index as i64 + 1));
        tx.insert_message(&message(id, account, sender, flags), sorted_at)?;
        tx.add_appearance(&Appearance {
            message: MessageId::from_raw(id),
            mailbox: MailboxId::from_raw("box-in"),
            uid: index as u32 + 1,
            deleted: false,
            needs_revalidation: false,
        })?;
    }
    Ok(())
}

/// Garnit le miroir du compte : boîtes et messages, compteurs de la réception à jour.
pub fn seed_mirror(db: &Database, account: &str) {
    db.write(|tx| {
        seed_mailboxes(tx, account)?;
        seed_messages(tx, account)?;
        tx.refresh_mailbox_counters(&MailboxId::from_raw("box-in"))
    })
    .unwrap();
}

/// Télécharge le corps de m1 (avec HTML) et de m3, et les indexe pour la recherche locale.
pub fn seed_bodies(db: &Database) {
    let html = "<p>Bonjour <b>monde</b></p>";
    let bodies = [
        ("m1", "Bonjour monde licorne", Some(html)),
        ("m3", "Rapport trimestriel", None),
    ];
    db.write(|tx| {
        for (id, text, html) in bodies {
            let id = MessageId::from_raw(id);
            tx.store_parsed(&id, text, html)?;
            tx.store_raw_source(&id, text.as_bytes())?;
            tx.set_content_state(&id, ContentState::BodyPresent)?;
            tx.index_message(&id, "Sujet", "bob@example.org", text)?;
        }
        Ok(())
    })
    .unwrap();
}

/// Regroupe m1 et m2 dans le fil `t-r1` et m3 dans le fil `t-r2`.
pub fn seed_threads(db: &Database, account: &str) {
    let account = AccountId::from_raw(account);
    db.write(|tx| {
        for (id, key) in [("m1", "r1"), ("m2", "r1"), ("m3", "r2")] {
            let candidate = ThreadId::from_raw(format!("t-{key}"));
            let thread = tx.upsert_thread(&account, key, "Sujet", &candidate)?;
            tx.set_thread(&MessageId::from_raw(id), Some(&thread))?;
        }
        Ok(())
    })
    .unwrap();
}
