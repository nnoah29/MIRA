//! Base volumineuse pour les mesures de performance : insertion en masse dans une seule transaction.

use mira_core::storage::Database;
use mira_vocabulary::Timestamp;

use crate::support_mirror::fixtures::{appearance, sample_message};
use crate::support_mirror::seeded::database_with_two_accounts;

/// Base avec `count` messages répartis sur acc-1/box-1 et acc-2/box-2 ; un sur quatre non lu, un sur dix suivi
pub fn database_with_bulk_messages(count: u32) -> Database {
    let db = database_with_two_accounts();
    db.write(|tx| {
        for index in 1..=count {
            let (account, mailbox) = match index % 2 {
                0 => ("acc-1", "box-1"),
                _ => ("acc-2", "box-2"),
            };
            let id = format!("bulk-{index}");
            let mut message = sample_message(&id, account);
            message.flags.seen = index % 4 != 0;
            message.flags.flagged = index % 10 == 0;
            let date = Timestamp::from_millis(i64::from(index) * 1_000);
            tx.insert_message(&message, date)?;
            tx.add_appearance(&appearance(&id, mailbox, index))?;
        }
        Ok(())
    })
    .unwrap();
    db
}
