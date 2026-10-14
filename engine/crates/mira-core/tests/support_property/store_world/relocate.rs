//! Déplacements entre boîtes et renumérotation d'une boîte par le serveur (revalidation des UID).

use mira_vocabulary::MessageId;

use super::{BOXES_PER_ACCOUNT, StoreWorld, mailbox_id};

impl StoreWorld {
    /// Déplace le message de sa première boîte vers une boîte de son compte
    pub(super) fn apply_move(&mut self, message: usize, to: usize) {
        let Some(index) = self.model.pick(message) else {
            return;
        };
        let target = self.model.messages[index].account * BOXES_PER_ACCOUNT + to;
        let Some(&source) = self.model.messages[index].appearances.keys().next() else {
            return;
        };
        if source == target {
            return;
        }
        self.place(index, target);
        let id = self.model.messages[index].id.clone();
        self.db
            .write(|tx| tx.remove_appearance(&id, &mailbox_id(source)))
            .unwrap();
        self.model.messages[index].appearances.remove(&source);
    }

    /// Le serveur renumérote la boîte : seules les apparitions du masque sont revalidées, les autres sont purgées
    pub(super) fn apply_renumber(&mut self, mailbox: usize, keep_mask: u8) {
        let members = self.model.members(mailbox);
        let (kept, dropped) = split_by_mask(&members, keep_mask);
        let purged = self.revalidate_and_purge(mailbox, &kept);
        let lost = self.update_model_after_renumber(mailbox, &kept, &dropped);
        assert_eq!(sorted(purged), sorted(lost), "orphelins de la purge");
    }

    /// Confirme les apparitions gardées sous des UID en ordre inverse, purge le reste et supprime les orphelins
    fn revalidate_and_purge(&self, mailbox: usize, kept: &[(u32, usize)]) -> Vec<MessageId> {
        let id_of = |index: usize| self.model.messages[index].id.clone();
        let mailbox = mailbox_id(mailbox);
        self.db
            .write(|tx| {
                tx.mark_mailbox_for_revalidation(&mailbox)?;
                for (uid, &(_, index)) in (1u32..).zip(kept.iter().rev()) {
                    tx.revalidate_appearance(&id_of(index), &mailbox, uid)?;
                }
                let orphans = tx.purge_unrevalidated(&mailbox)?;
                for orphan in &orphans {
                    tx.delete_message(orphan)?;
                }
                Ok(orphans)
            })
            .unwrap()
    }

    /// Met le modèle à jour ; rend les messages qui n'ont plus aucune apparition
    fn update_model_after_renumber(
        &mut self,
        mailbox: usize,
        kept: &[(u32, usize)],
        dropped: &[usize],
    ) -> Vec<MessageId> {
        for (uid, &(_, index)) in (1u32..).zip(kept.iter().rev()) {
            if let Some(found) = self.model.messages[index].appearances.get_mut(&mailbox) {
                found.uid = uid;
            }
        }
        let mut lost = Vec::new();
        for &index in dropped {
            self.model.messages[index].appearances.remove(&mailbox);
            if self.model.messages[index].appearances.is_empty() {
                lost.push(self.model.messages[index].id.clone());
            }
        }
        self.model
            .messages
            .retain(|message| !lost.contains(&message.id));
        self.model.next_uid[mailbox] = u32::try_from(kept.len()).unwrap() + 1;
        lost
    }
}

/// Sépare les membres (UID, position) entre gardés et abandonnés selon les bits du masque
fn split_by_mask(members: &[(u32, usize)], mask: u8) -> (Vec<(u32, usize)>, Vec<usize>) {
    let mut kept = Vec::new();
    let mut dropped = Vec::new();
    for (rank, &(uid, index)) in members.iter().enumerate() {
        if (mask >> (rank % 8)) & 1 == 1 {
            kept.push((uid, index));
        } else {
            dropped.push(index);
        }
    }
    (kept, dropped)
}

/// Identifiants triés, pour comparer des ensembles
fn sorted(mut ids: Vec<MessageId>) -> Vec<MessageId> {
    ids.sort_by(|left, right| left.as_str().cmp(right.as_str()));
    ids
}
