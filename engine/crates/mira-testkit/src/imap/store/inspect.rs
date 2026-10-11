//! API publique de lecture du magasin pour les tests.

use super::handle::FakeMailStore;

/// Copie d'un message tel que le serveur le conserve.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FakeMessageView {
    pub uid: u32,
    pub flags: Vec<String>,
    pub raw: Vec<u8>,
    pub modseq: u64,
    pub gm_msgid: u64,
    pub gm_thrid: u64,
}

impl FakeMailStore {
    /// Messages d'une boîte dans l'ordre des UID
    pub fn messages(&self, user: &str, mailbox: &str) -> Vec<FakeMessageView> {
        self.read(|data| {
            let entries = data
                .mailbox(user, mailbox)
                .map(|found| found.entries.as_slice());
            entries
                .unwrap_or(&[])
                .iter()
                .map(|entry| FakeMessageView {
                    uid: entry.uid,
                    flags: entry.flags.clone(),
                    raw: entry.message.raw.clone(),
                    modseq: entry.modseq,
                    gm_msgid: entry.message.gm_msgid,
                    gm_thrid: entry.message.gm_thrid,
                })
                .collect()
        })
    }

    /// UIDVALIDITY courant d'une boîte
    pub fn uidvalidity(&self, user: &str, mailbox: &str) -> Option<u32> {
        self.read(|data| data.mailbox(user, mailbox).map(|found| found.uidvalidity))
    }

    /// HIGHESTMODSEQ courant d'une boîte
    pub fn highest_modseq(&self, user: &str, mailbox: &str) -> Option<u64> {
        self.read(|data| {
            data.mailbox(user, mailbox)
                .map(|found| found.highest_modseq)
        })
    }
}
