//! Garnissage du miroir : boîte, messages en-têtes seuls, corps présents.

use mira_core::storage::Database;
use mira_vocabulary::{
    AccountId, ContentState, MailboxId, MailboxRole, MessageId, MessageSummary, ThreadId, Timestamp,
};

use crate::support_mirror::fixtures::{appearance, sample_mailbox, sample_message};

/// Crée la boîte INBOX locale du compte
pub fn add_inbox(db: &Database, account: &AccountId) -> MailboxId {
    let inbox = sample_mailbox("box-1", account.as_str(), "INBOX", MailboxRole::Inbox);
    db.write(|tx| tx.upsert_mailbox(&inbox)).unwrap()
}

/// Message en-têtes seuls de sujet donné, daté de `millis`, sous un UID de la boîte
pub fn add_message(
    db: &Database,
    account: &AccountId,
    mailbox: &MailboxId,
    id: &str,
    uid: u32,
    millis: i64,
) -> MessageId {
    let mut summary: MessageSummary = sample_message(id, account.as_str());
    summary.subject = format!("Sujet {id}");
    db.write(|tx| {
        tx.insert_message(&summary, Timestamp::from_millis(millis))?;
        tx.add_appearance(&appearance(id, mailbox.as_str(), uid))
    })
    .unwrap();
    MessageId::from_raw(id)
}

/// Range un corps déjà téléchargé : texte, HTML éventuel, source, index de recherche
pub fn add_body(db: &Database, id: &MessageId, text: &str, html: Option<&str>) {
    db.write(|tx| {
        tx.store_parsed(id, text, html)?;
        tx.store_raw_source(id, text.as_bytes())?;
        tx.set_content_state(id, ContentState::BodyPresent)?;
        tx.index_message(id, "Sujet", "bob@example.org", text)
    })
    .unwrap();
}

/// Regroupe m1 et m2 dans le fil `t-r1` et m3 dans le fil `t-r2`
pub fn link_threads(db: &Database, account: &AccountId) {
    db.write(|tx| {
        for (message, key) in [("m1", "r1"), ("m2", "r1"), ("m3", "r2")] {
            let candidate = ThreadId::from_raw(format!("t-{key}"));
            let thread = tx.upsert_thread(account, key, "S", &candidate)?;
            tx.set_thread(&MessageId::from_raw(message), Some(&thread))?;
        }
        Ok(())
    })
    .unwrap();
}

/// Crée la boîte Archive locale du compte
pub fn add_archive(db: &Database, account: &AccountId) -> MailboxId {
    let archive = sample_mailbox("box-2", account.as_str(), "Archive", MailboxRole::Archive);
    db.write(|tx| tx.upsert_mailbox(&archive)).unwrap()
}
