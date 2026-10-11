//! Erreur de commande convertie en réponse NO ou BAD.

use super::reply::Reply;
use crate::imap::parser::error::ParseError;

/// Échec d'une commande, renvoyé au client en `NO` ou `BAD`.
#[derive(Debug, Clone)]
pub(crate) struct ImapError {
    pub status: &'static str,
    pub text: String,
}

/// Résultat d'un gestionnaire de commande.
pub(crate) type HandlerResult = Result<Reply, ImapError>;

impl ImapError {
    /// Refus de la commande (NO)
    pub(crate) fn no(text: &str) -> Self {
        Self {
            status: "NO",
            text: text.to_string(),
        }
    }

    /// Commande invalide dans ce contexte (BAD)
    pub(crate) fn bad(text: &str) -> Self {
        Self {
            status: "BAD",
            text: text.to_string(),
        }
    }

    /// Réponse étiquetée correspondant à l'échec
    pub(crate) fn into_reply(self, tag: &str) -> Reply {
        Reply::default().tagged(tag, self.status, &self.text)
    }
}

impl From<ParseError> for ImapError {
    /// Une erreur de syntaxe devient un BAD
    fn from(error: ParseError) -> Self {
        Self::bad(&error.message)
    }
}
