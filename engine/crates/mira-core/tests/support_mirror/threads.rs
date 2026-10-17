//! Base de test avec un fil par message.

use mira_core::storage::Database;
use mira_vocabulary::{AccountId, MessageId, ThreadId};

use super::seeded::database_with_messages;

/// Trois messages (msg-1 à msg-3) dans box-1, chacun dans son fil (t-0, t-1, t-2)
pub fn database_with_three_threads() -> Database {
    let db = database_with_messages(3);
    let acc = AccountId::from_raw("acc-1");
    db.write(|tx| {
        for (index, name) in ["msg-1", "msg-2", "msg-3"].into_iter().enumerate() {
            let thread = ThreadId::from_raw(format!("t-{index}"));
            let id = tx.upsert_thread(&acc, &format!("r{index}"), "S", &thread)?;
            tx.set_thread(&MessageId::from_raw(name), Some(&id))?;
        }
        Ok(())
    })
    .unwrap();
    db
}
