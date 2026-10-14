//! Miroir réel (base en mémoire) et modèle de référence, avancés ensemble opération par opération.

mod ingest;
mod messages;
mod relocate;

use mira_core::storage::Database;
use mira_vocabulary::{AccountId, MailboxId, MailboxRole};

use super::model::{ACCOUNTS, BOX_COUNT, BOXES_PER_ACCOUNT, Model, account_of_box};
use super::operations::StoreOp;
use crate::support::accounts::sample_account;
use crate::support_mirror::fixtures::sample_mailbox;

/// Base réelle et modèle attendu
#[derive(Debug)]
pub struct StoreWorld {
    pub db: Database,
    pub model: Model,
}

/// Identifiant de compte de numéro donné
pub fn account_id(account: usize) -> AccountId {
    AccountId::from_raw(format!("acc-{account}"))
}

/// Identifiant de boîte de numérotation globale donnée
pub fn mailbox_id(mailbox: usize) -> MailboxId {
    MailboxId::from_raw(format!("box-{mailbox}"))
}

impl StoreWorld {
    /// Deux comptes de trois boîtes chacun (réception, archive, travail), sans message
    pub fn new() -> Self {
        let db = Database::open_in_memory().unwrap();
        db.write(|tx| {
            for account in 0..ACCOUNTS {
                tx.insert_account(&sample_account(account_id(account).as_str()))?;
            }
            for mailbox in 0..BOX_COUNT {
                let names = ["INBOX", "Archive", "Work"];
                let roles = [MailboxRole::Inbox, MailboxRole::Archive, MailboxRole::Other];
                let local = mailbox % BOXES_PER_ACCOUNT;
                let account = account_id(account_of_box(mailbox));
                let id = mailbox_id(mailbox);
                tx.upsert_mailbox(&sample_mailbox(
                    id.as_str(),
                    account.as_str(),
                    names[local],
                    roles[local],
                ))?;
            }
            Ok(())
        })
        .unwrap();
        Self {
            db,
            model: Model::new(),
        }
    }

    /// Applique l'opération à la base et au modèle
    pub fn apply(&mut self, operation: &StoreOp) {
        match *operation {
            StoreOp::Ingest {
                account,
                mailbox,
                stable,
            } => self.apply_ingest(account, mailbox, stable),
            StoreOp::Label { message, mailbox } => self.apply_label(message, mailbox),
            StoreOp::Move { message, to } => self.apply_move(message, to),
            StoreOp::Unlabel { message, mailbox } => self.apply_unlabel(message, mailbox),
            StoreOp::Renumber { mailbox, keep_mask } => self.apply_renumber(mailbox, keep_mask),
            StoreOp::SetFlags { message, bits } => self.apply_flags(message, bits),
            StoreOp::Trash {
                message,
                mailbox,
                deleted,
            } => self.apply_trash(message, mailbox, deleted),
            StoreOp::EndOfPass { account } => self.apply_end_of_pass(account),
            StoreOp::Delete { message } => self.apply_delete(message),
        }
    }

    /// Termine la passe de chaque compte : les messages sans apparition disparaissent
    pub fn finish_passes(&mut self) {
        for account in 0..ACCOUNTS {
            self.apply(&StoreOp::EndOfPass { account });
        }
    }
}
