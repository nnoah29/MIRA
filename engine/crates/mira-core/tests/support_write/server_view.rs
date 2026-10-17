//! Lecture de ce que le faux serveur IMAP contient, pour vérifier l'effet du rejeu.

use super::USER;
use super::fixture::Fixture;

impl Fixture {
    /// UID des messages de la boîte du serveur, dans l'ordre
    pub fn server_uids(&self, mailbox: &str) -> Vec<u32> {
        let views = self.store.messages(USER, mailbox);
        views.iter().map(|view| view.uid).collect()
    }

    /// Indicateurs d'un message de la boîte du serveur
    pub fn server_flags(&self, mailbox: &str, uid: u32) -> Vec<String> {
        let views = self.store.messages(USER, mailbox);
        let found = views.into_iter().find(|view| view.uid == uid);
        found.map(|view| view.flags).unwrap_or_default()
    }

    /// Dit si le message est marqué lu sur le serveur
    pub fn server_seen(&self, mailbox: &str, uid: u32) -> bool {
        self.server_flags(mailbox, uid)
            .iter()
            .any(|flag| flag.eq_ignore_ascii_case("\\Seen"))
    }

    /// Identifiants des boîtes du miroir où le message est visible
    pub fn boxes_of(&self, id: &str) -> Vec<String> {
        let boxes = self.message(id).mailboxes;
        boxes
            .iter()
            .map(|mailbox| mailbox.as_str().to_string())
            .collect()
    }
}
