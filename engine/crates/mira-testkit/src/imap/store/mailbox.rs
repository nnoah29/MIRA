//! Opérations sur une boîte.

use std::sync::Arc;

use super::data::{Entry, Expunged, Mailbox, MessageBody};

impl Mailbox {
    /// Crée une boîte vide avec son UIDVALIDITY
    pub(crate) fn new(uidvalidity: u32, special_use: Option<&str>) -> Self {
        Self {
            special_use: special_use.map(str::to_string),
            uidvalidity,
            uidnext: 1,
            highest_modseq: 1,
            entries: Vec::new(),
            expunged: Vec::new(),
        }
    }

    /// Incrémente le HIGHESTMODSEQ et le renvoie
    pub(crate) fn bump_modseq(&mut self) -> u64 {
        self.highest_modseq += 1;
        self.highest_modseq
    }

    /// Ajoute un message et renvoie son UID
    pub(crate) fn append(
        &mut self,
        flags: Vec<String>,
        internal_date: i64,
        message: Arc<MessageBody>,
    ) -> u32 {
        let modseq = self.bump_modseq();
        let uid = self.uidnext;
        self.uidnext += 1;
        self.entries.push(Entry {
            uid,
            modseq,
            flags,
            internal_date,
            message,
        });
        uid
    }

    /// UID des messages dans l'ordre des numéros de séquence
    pub(crate) fn uids(&self) -> Vec<u32> {
        self.entries.iter().map(|entry| entry.uid).collect()
    }

    /// Supprime les messages indiqués et mémorise leur disparition
    pub(crate) fn remove_uids(&mut self, uids: &[u32]) -> Vec<u32> {
        let doomed: Vec<u32> = self
            .uids()
            .into_iter()
            .filter(|uid| uids.contains(uid))
            .collect();
        for uid in &doomed {
            let modseq = self.bump_modseq();
            self.expunged.push(Expunged { uid: *uid, modseq });
        }
        self.entries.retain(|entry| !doomed.contains(&entry.uid));
        doomed
    }

    /// Entrée portant cet UID
    pub(crate) fn entry_mut(&mut self, uid: u32) -> Option<&mut Entry> {
        self.entries.iter_mut().find(|entry| entry.uid == uid)
    }
}
