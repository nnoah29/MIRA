//! Message vu sur le serveur, prêt à être reconnu ou créé dans le miroir.

use mira_vocabulary::{MailboxId, Provider, ServerMessageTranslation};

use crate::imap::FetchedHeaders;
use crate::mime::{HeaderSummary, parse_headers};

/// En-têtes d'un message du serveur avec ce que le provider et l'analyse en déduisent.
#[derive(Debug, Clone)]
pub struct Incoming {
    pub mailbox: MailboxId,
    pub validity: u32,
    pub fetched: FetchedHeaders,
    pub parsed: HeaderSummary,
    pub translation: ServerMessageTranslation,
}

impl Incoming {
    /// Analyse les en-têtes et traduit les attributs propres au service.
    pub fn new(
        provider: &dyn Provider,
        mailbox: &MailboxId,
        validity: u32,
        fetched: FetchedHeaders,
    ) -> Self {
        let parsed = parse_headers(&fetched.header_bytes);
        let translation = provider.translate_attributes(&fetched.extra);
        Self {
            mailbox: mailbox.clone(),
            validity,
            fetched,
            parsed,
            translation,
        }
    }

    /// UID du message dans sa boîte.
    pub fn uid(&self) -> u32 {
        self.fetched.uid
    }

    /// Clé de correspondance « boîte/validité/uid » (D-13).
    pub fn uid_key(&self) -> String {
        format!("{}/{}/{}", self.mailbox, self.validity, self.fetched.uid)
    }
}
