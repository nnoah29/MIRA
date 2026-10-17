//! Courrier déposé sur le faux serveur IMAP et état des messages tel que le serveur le conserve.

use super::sb_world::SbWorld;

impl SbWorld {
    /// Courrier brut de sujet donné, avec un `Message-ID` dérivé du sujet.
    pub fn raw_mail(user: &str, subject: &str) -> Vec<u8> {
        let id = subject.replace(' ', "-");
        format!(
            "From: Bob <bob@example.org>\r\nTo: {user}\r\nSubject: {subject}\r\nMessage-ID: <{id}@example.org>\r\nDate: Mon, 01 Jan 2024 10:00:00 +0000\r\n\r\nCorps de {subject}\r\n"
        )
        .into_bytes()
    }

    /// Dépose un courrier sur le serveur de l'utilisateur, dans la boîte donnée, et rend son UID.
    pub fn deliver_subject(&self, user: &str, mailbox: &str, subject: &str) -> u32 {
        let raw = Self::raw_mail(user, subject);
        self.store.deliver(user, mailbox, &raw, &[])
    }

    /// Dit si le serveur porte le drapeau `\Seen` sur ce message.
    pub fn server_seen(&self, user: &str, mailbox: &str, uid: u32) -> bool {
        let views = self.store.messages(user, mailbox);
        let found = views.iter().find(|view| view.uid == uid).unwrap();
        found
            .flags
            .iter()
            .any(|flag| flag.eq_ignore_ascii_case("\\Seen"))
    }

    /// UID des messages d'une boîte du serveur, dans l'ordre.
    pub fn server_uids(&self, user: &str, mailbox: &str) -> Vec<u32> {
        let views = self.store.messages(user, mailbox);
        views.iter().map(|view| view.uid).collect()
    }

    /// Compteurs (total, non lus) de la boîte de ce nom, tels que `mailbox list` les rend.
    pub async fn counters_of(&self, account: &str, name: &str) -> (u64, u64) {
        let data = self.run_ok(&["mailbox", "list", account]).await;
        let boxes = data["data"].as_array().unwrap();
        let found = boxes.iter().find(|item| item["name"] == name).unwrap();
        (
            found["total"].as_u64().unwrap(),
            found["unread"].as_u64().unwrap(),
        )
    }

    /// Identifiant interne du message de ce sujet dans une boîte du miroir.
    pub async fn message_id_of(&self, mailbox: &str, subject: &str) -> String {
        let messages = self.messages_of(mailbox).await;
        let found = messages.iter().find(|item| item["subject"] == subject);
        found.unwrap()["id"].as_str().unwrap().to_string()
    }
}
