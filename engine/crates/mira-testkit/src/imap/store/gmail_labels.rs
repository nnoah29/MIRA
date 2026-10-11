//! Libellés Gmail : ajouter ou retirer un message d'une boîte.

use std::sync::Arc;

use super::data::{Entry, StoreData};
use super::entry_flags::FlagMode;

impl StoreData {
    /// Ajoute ou retire le message des boîtes désignées par les libellés
    pub(crate) fn change_labels(
        &mut self,
        user: &str,
        mailbox: &str,
        uid: u32,
        mode: FlagMode,
        labels: &[String],
    ) {
        let found = self
            .mailbox(user, mailbox)
            .and_then(|found| found.entries.iter().find(|entry| entry.uid == uid))
            .cloned();
        let Some(entry) = found else {
            return;
        };
        for label in labels {
            let name = self
                .users
                .get(user)
                .map_or_else(|| label.clone(), |owner| owner.mailbox_for_label(label));
            match mode {
                FlagMode::Remove => self.remove_message_from(user, &name, entry.message.gm_msgid),
                FlagMode::Add | FlagMode::Replace => self.add_message_to(user, &name, &entry),
            }
        }
    }

    /// Ajoute le message à la boîte s'il n'y est pas déjà
    fn add_message_to(&mut self, user: &str, name: &str, entry: &Entry) {
        let target = self.mailbox_or_create(user, name);
        let present = target
            .entries
            .iter()
            .any(|known| known.message.gm_msgid == entry.message.gm_msgid);
        if !present {
            let message = Arc::clone(&entry.message);
            target.append(entry.flags.clone(), entry.internal_date, message);
        }
    }

    /// Retire de la boîte les messages portant cet identifiant Gmail
    fn remove_message_from(&mut self, user: &str, name: &str, gm_msgid: u64) {
        if let Some(target) = self.mailbox_mut(user, name) {
            let doomed: Vec<u32> = target
                .entries
                .iter()
                .filter(|entry| entry.message.gm_msgid == gm_msgid)
                .map(|entry| entry.uid)
                .collect();
            target.remove_uids(&doomed);
        }
    }
}
