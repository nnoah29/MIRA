//! Messages ajoutés directement au miroir pour l'aperçu des règles.

use mira_core::storage::Database;
use mira_vocabulary::{AccountId, Timestamp};

use crate::support_mirror::fixtures::{appearance, sample_message};

/// Ajoute `count` messages « Lot n » (n croissant = plus récent) en réception, et un dernier en Corbeille seulement
pub fn seed_lots(db: &Database, account: &AccountId, count: u32) {
    db.write(|tx| {
        for n in 0..=count {
            let id = format!("lot-{n}");
            let mut message = sample_message(&id, account.as_str());
            message.subject = format!("Lot {n}");
            tx.insert_message(&message, Timestamp::from_millis(10_000 + i64::from(n)))?;
            let mailbox = if n == count { "box-trash" } else { "box-in" };
            tx.add_appearance(&appearance(&id, mailbox, 100 + n))?;
        }
        Ok(())
    })
    .unwrap();
}
