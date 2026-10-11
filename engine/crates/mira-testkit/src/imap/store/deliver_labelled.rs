//! Livraison d'un message présent sous plusieurs libellés, sous un nom neutre pour les tests du cœur.

use super::handle::FakeMailStore;

impl FakeMailStore {
    /// Livre le même message dans plusieurs boîtes (libellés) avec des identifiants de message et de fil imposés
    pub fn deliver_with_labels(
        &self,
        user: &str,
        mailboxes: &[&str],
        raw: &[u8],
        message_id: u64,
        thread_id: u64,
    ) -> Vec<u32> {
        self.deliver_gmail(user, mailboxes, raw, message_id, thread_id)
    }
}
