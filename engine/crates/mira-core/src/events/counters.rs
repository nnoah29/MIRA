//! Annonce des compteurs de boîtes qui ont changé depuis la dernière annonce, avant l'événement qui les explique.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};

use mira_vocabulary::{AccountId, EventCategory, EventKind, Mailbox, MailboxId, MiraError};

use super::bus::EventBus;
use crate::storage::Database;

/// Derniers compteurs annoncés (ou connus à l'ouverture) de chaque boîte : total, non lus.
#[derive(Debug, Clone, Default)]
pub(super) struct Announced {
    seen: Arc<Mutex<HashMap<MailboxId, (u64, u64)>>>,
}

impl Announced {
    /// Prend le verrou en ignorant un empoisonnement
    fn lock(&self) -> MutexGuard<'_, HashMap<MailboxId, (u64, u64)>> {
        self.seen
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Retient les compteurs d'une boîte
    pub(super) fn remember(&self, mailbox: &MailboxId, total: u64, unread: u64) {
        self.lock().insert(mailbox.clone(), (total, unread));
    }

    /// Mémorise les compteurs actuels de toutes les boîtes, pour ne rien annoncer à l'ouverture
    pub(super) fn seed(&self, db: &Database) {
        let boxes = db.read(|tx| {
            let mut all = Vec::new();
            for account in tx.list_accounts()? {
                all.extend(tx.list_mailboxes(&account.id)?);
            }
            Ok(all)
        });
        for mailbox in boxes.unwrap_or_default() {
            self.remember(&mailbox.id, mailbox.total, mailbox.unread);
        }
    }

    /// Boîtes dont les compteurs diffèrent de ce qui a été annoncé
    fn changed(&self, boxes: Vec<Mailbox>) -> Vec<Mailbox> {
        let seen = self.lock();
        let differs = |b: &Mailbox| seen.get(&b.id) != Some(&(b.total, b.unread));
        boxes.into_iter().filter(differs).collect()
    }
}

/// Vrai si l'événement peut avoir changé des compteurs (message, fil, action, envoi, règle)
pub(super) fn may_change_counters(kind: &EventKind) -> bool {
    if matches!(kind, EventKind::AttachmentProgress { .. }) {
        return false;
    }
    let category = kind.category();
    let relevant = [
        EventCategory::Message,
        EventCategory::Thread,
        EventCategory::Action,
        EventCategory::Send,
        EventCategory::Rule,
    ];
    relevant.contains(&category)
}

impl EventBus {
    /// Annonce les compteurs du compte qui ont changé depuis la dernière annonce
    pub(super) fn announce_changed_counters(&self, account: &AccountId) -> Result<(), MiraError> {
        let boxes = self.db.read(|tx| tx.list_mailboxes(account))?;
        for mailbox in self.announced.changed(boxes) {
            self.emit_plain(EventKind::MailboxCountersUpdated {
                account: mailbox.account,
                mailbox: mailbox.id,
                total: mailbox.total,
                unread: mailbox.unread,
            })?;
        }
        Ok(())
    }
}
