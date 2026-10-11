//! État d'une transaction SMTP : utilisateur authentifié, expéditeur, destinataires.

use crate::smtp::mail::ReceivedMail;

/// Données accumulées entre MAIL FROM et la fin de DATA.
#[derive(Debug, Default)]
pub(crate) struct Transaction {
    pub(crate) user: Option<String>,
    pub(crate) from: Option<String>,
    pub(crate) recipients: Vec<String>,
}

impl Transaction {
    /// Abandonne l'enveloppe en cours en gardant l'authentification
    pub(crate) fn reset(&mut self) {
        self.from = None;
        self.recipients.clear();
    }

    /// Indique si DATA est permis : un expéditeur et au moins un destinataire
    pub(crate) fn is_ready_for_data(&self) -> bool {
        self.from.is_some() && !self.recipients.is_empty()
    }

    /// Transforme l'enveloppe et les données en message reçu puis réinitialise
    pub(crate) fn take_mail(&mut self, data: Vec<u8>) -> ReceivedMail {
        let from = self.from.take().unwrap_or_default();
        let recipients = std::mem::take(&mut self.recipients);
        ReceivedMail {
            from,
            recipients,
            data,
        }
    }
}
