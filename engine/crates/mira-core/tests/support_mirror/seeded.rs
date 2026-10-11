//! Bases de test déjà garnies : comptes, boîtes et messages.

use mira_core::storage::Database;
use mira_vocabulary::{MailboxRole, Timestamp};

use super::fixtures::{appearance, sample_mailbox, sample_message};
use crate::support::accounts::sample_account;

/// Base avec les comptes acc-1 et acc-2, chacun avec une boîte (box-1, box-2)
pub fn database_with_two_accounts() -> Database {
    let db = Database::open_in_memory().unwrap();
    db.write(|tx| {
        tx.insert_account(&sample_account("acc-1"))?;
        tx.insert_account(&sample_account("acc-2"))?;
        tx.upsert_mailbox(&sample_mailbox(
            "box-1",
            "acc-1",
            "INBOX",
            MailboxRole::Inbox,
        ))?;
        tx.upsert_mailbox(&sample_mailbox(
            "box-2",
            "acc-2",
            "INBOX",
            MailboxRole::Inbox,
        ))?;
        Ok(())
    })
    .unwrap();
    db
}

/// Base avec `count` messages (msg-1…) dans box-1, de date de tri croissante, UID 1…
pub fn database_with_messages(count: u32) -> Database {
    let db = database_with_two_accounts();
    db.write(|tx| {
        for index in 1..=count {
            let id = format!("msg-{index}");
            let date = Timestamp::from_millis(i64::from(index) * 1_000);
            tx.insert_message(&sample_message(&id, "acc-1"), date)?;
            tx.add_appearance(&appearance(&id, "box-1", index))?;
        }
        Ok(())
    })
    .unwrap();
    db
}
