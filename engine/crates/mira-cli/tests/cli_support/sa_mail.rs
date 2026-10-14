//! Courrier déposé sur le faux serveur par les scénarios : sujet, en-têtes de fil, ancienneté.

use std::time::{SystemTime, UNIX_EPOCH};

use super::sa_world::SaWorld;

/// Courrier brut de Bob pour l'utilisateur, avec un `Message-ID` dérivé du sujet.
pub fn raw_mail(user: &str, subject: &str, extra_headers: &str) -> Vec<u8> {
    let id = subject.replace(' ', "-");
    format!(
        "From: Bob <bob@example.org>\r\nTo: {user}\r\nSubject: {subject}\r\nMessage-ID: <{id}@example.org>\r\n{extra_headers}\r\nCorps de {subject}\r\n"
    )
    .into_bytes()
}

/// Courrier à pièce jointe `notes.txt` (douze octets) dont le sujet est donné.
pub fn mail_with_notes(user: &str, subject: &str) -> Vec<u8> {
    let id = subject.replace(' ', "-");
    format!(
        "From: Bob <bob@example.org>\r\nTo: {user}\r\nSubject: {subject}\r\nMessage-ID: <{id}@example.org>\r\nMIME-Version: 1.0\r\nContent-Type: multipart/mixed; boundary=\"B\"\r\n\r\n--B\r\nContent-Type: text/plain\r\n\r\nVoir notes\r\n--B\r\nContent-Type: text/plain; name=\"notes.txt\"\r\nContent-Disposition: attachment; filename=\"notes.txt\"\r\n\r\nnotes: douze\r\n--B--\r\n"
    )
    .into_bytes()
}

impl SaWorld {
    /// Dépose un courrier dans la boîte du serveur et rend son UID.
    pub fn deliver(&self, mailbox: &str, subject: &str, extra_headers: &str) -> u32 {
        let raw = raw_mail(&self.user, subject, extra_headers);
        self.bench.store.deliver(&self.user, mailbox, &raw, &[])
    }

    /// Sujets des courriers que le serveur garde dans cette boîte, dans l'ordre des UID.
    pub fn server_subjects(&self, mailbox: &str) -> Vec<String> {
        let views = self.bench.store.messages(&self.user, mailbox);
        let subject = |raw: &[u8]| {
            let text = String::from_utf8_lossy(raw).into_owned();
            let line = text.lines().find(|line| line.starts_with("Subject: "));
            line.unwrap_or_default()
                .trim_start_matches("Subject: ")
                .to_string()
        };
        views.iter().map(|view| subject(&view.raw)).collect()
    }

    /// Dépose un courrier dont la date interne remonte à ce nombre de jours.
    pub fn deliver_aged(&self, mailbox: &str, subject: &str, days: i64) -> u32 {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        let date = now.as_secs() as i64 - days * 86_400;
        let raw = raw_mail(&self.user, subject, "");
        let store = &self.bench.store;
        store.deliver_dated(&self.user, mailbox, &raw, &[], date)
    }
}
