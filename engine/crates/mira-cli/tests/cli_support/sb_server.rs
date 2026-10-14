//! État des messages tel que le faux serveur IMAP les conserve, retrouvés par leur sujet.

use super::sb_world::SbWorld;

impl SbWorld {
    /// UID du message de ce sujet dans une boîte du serveur.
    pub fn server_uid_of(&self, user: &str, mailbox: &str, subject: &str) -> u32 {
        let wanted = format!("Subject: {subject}\r\n");
        let views = self.store.messages(user, mailbox);
        let found = views.iter().find(|view| {
            let text = String::from_utf8_lossy(&view.raw);
            text.contains(&wanted)
        });
        found.unwrap().uid
    }

    /// Dit si le serveur porte ce drapeau (nom IMAP, par exemple `\Flagged`) sur le message de ce sujet.
    pub fn server_has_flag(&self, user: &str, mailbox: &str, subject: &str, flag: &str) -> bool {
        let uid = self.server_uid_of(user, mailbox, subject);
        let views = self.store.messages(user, mailbox);
        let found = views.iter().find(|view| view.uid == uid).unwrap();
        found.flags.iter().any(|set| set.eq_ignore_ascii_case(flag))
    }
}
