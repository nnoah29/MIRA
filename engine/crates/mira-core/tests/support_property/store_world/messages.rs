//! Opérations sur un message entier : drapeaux, marque « supprimé » d'une apparition, fin de passe, suppression.

use mira_vocabulary::{Flags, MessageId};

use super::{StoreWorld, account_id, mailbox_id};

/// Drapeaux décrits par les quatre bits de poids faible
fn flags_from_bits(bits: u8) -> Flags {
    Flags {
        seen: bits & 1 != 0,
        flagged: bits & 2 != 0,
        answered: bits & 4 != 0,
        forwarded: bits & 8 != 0,
    }
}

impl StoreWorld {
    /// Remplace les drapeaux du message (jamais ceux d'une apparition)
    pub(super) fn apply_flags(&mut self, message: usize, bits: u8) {
        let Some(index) = self.model.pick(message) else {
            return;
        };
        let flags = flags_from_bits(bits);
        let id = self.model.messages[index].id.clone();
        self.db
            .write(|tx| tx.update_message_flags(&id, flags))
            .unwrap();
        self.model.messages[index].flags = flags;
    }

    /// Marque (ou démarque) l'apparition comme supprimée sur le serveur, si elle existe
    pub(super) fn apply_trash(&mut self, message: usize, mailbox: usize, deleted: bool) {
        let Some(index) = self.model.pick(message) else {
            return;
        };
        let id = self.model.messages[index].id.clone();
        let Some(found) = self.model.messages[index].appearances.get_mut(&mailbox) else {
            return;
        };
        found.deleted = deleted;
        self.db
            .write(|tx| tx.set_appearance_deleted(&id, &mailbox_id(mailbox), deleted))
            .unwrap();
    }

    /// Fin de passe d'un compte : les messages sans apparition sont supprimés du miroir
    pub(super) fn apply_end_of_pass(&mut self, account: usize) {
        let removed = self
            .db
            .write(|tx| tx.delete_orphan_messages(&account_id(account)))
            .unwrap();
        let orphans: Vec<MessageId> = self
            .model
            .messages
            .iter()
            .filter(|message| message.account == account && message.appearances.is_empty())
            .map(|message| message.id.clone())
            .collect();
        let mut left: Vec<&str> = removed.iter().map(MessageId::as_str).collect();
        let mut right: Vec<&str> = orphans.iter().map(MessageId::as_str).collect();
        left.sort_unstable();
        right.sort_unstable();
        assert_eq!(left, right, "messages sans apparition");
        self.model
            .messages
            .retain(|message| !orphans.contains(&message.id));
    }

    /// Supprime définitivement un message du miroir
    pub(super) fn apply_delete(&mut self, message: usize) {
        let Some(index) = self.model.pick(message) else {
            return;
        };
        let removed = self.model.messages.remove(index);
        self.db.write(|tx| tx.delete_message(&removed.id)).unwrap();
    }
}
