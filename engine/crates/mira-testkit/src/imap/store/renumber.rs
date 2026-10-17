//! Renumérotation d'une boîte (nouvel UIDVALIDITY).

use super::data::Mailbox;
use super::handle::FakeMailStore;
use super::user::take_validity;

impl Mailbox {
    /// Réattribue les UID à partir de 1 avec un nouvel UIDVALIDITY
    pub(crate) fn renumber(&mut self, validity: u32) {
        for (index, entry) in self.entries.iter_mut().enumerate() {
            entry.uid = index as u32 + 1;
        }
        self.uidnext = self.entries.len() as u32 + 1;
        self.uidvalidity = validity;
        self.expunged.clear();
        self.bump_modseq();
    }
}

impl FakeMailStore {
    /// Donne à la boîte un nouvel UIDVALIDITY et de nouveaux UID
    pub fn renumber(&self, user: &str, mailbox: &str) {
        self.write(|data| {
            let validity = take_validity(&mut data.validity_counter);
            if let Some(found) = data.mailbox_mut(user, mailbox) {
                found.renumber(validity);
            }
        });
    }
}
