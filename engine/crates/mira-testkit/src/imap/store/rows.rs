//! Lignes de messages prêtes à être rendues par FETCH.

use std::sync::Arc;

use super::data::{Entry, MessageBody, StoreData};

/// Copie d'un message d'une boîte au moment de la lecture.
#[derive(Debug, Clone)]
pub(crate) struct MessageRow {
    pub seq: u32,
    pub uid: u32,
    pub flags: Vec<String>,
    pub modseq: u64,
    pub internal_date: i64,
    pub message: Arc<MessageBody>,
    pub labels: Vec<String>,
}

/// Copie une entrée avec son numéro de séquence
fn row_of(seq: usize, entry: &Entry) -> MessageRow {
    MessageRow {
        seq: seq as u32 + 1,
        uid: entry.uid,
        flags: entry.flags.clone(),
        modseq: entry.modseq,
        internal_date: entry.internal_date,
        message: Arc::clone(&entry.message),
        labels: Vec::new(),
    }
}

impl StoreData {
    /// Lignes des UID demandés ; les libellés Gmail ne sont calculés que sur demande
    pub(crate) fn rows(
        &self,
        user: &str,
        mailbox: &str,
        uids: &[u32],
        labels: bool,
    ) -> Vec<MessageRow> {
        let Some(found) = self.mailbox(user, mailbox) else {
            return Vec::new();
        };
        let owner = self.users.get(user);
        let mut rows = Vec::new();
        for (index, entry) in found.entries.iter().enumerate() {
            if !uids.contains(&entry.uid) {
                continue;
            }
            let mut row = row_of(index, entry);
            if let (true, Some(owner)) = (labels, owner) {
                row.labels = owner.labels_of(entry.message.gm_msgid);
            }
            rows.push(row);
        }
        rows
    }
}
