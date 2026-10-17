//! Accès en écriture à la boîte sélectionnée.

use super::state::SessionState;
use crate::imap::response::error::ImapError;

impl SessionState {
    /// Utilisateur et boîte sélectionnée, refusés si la boîte est en lecture seule
    pub(crate) fn writable_target(&self) -> Result<(String, String), ImapError> {
        if self.require_selected()?.read_only {
            return Err(ImapError::no("[READ-ONLY] Mailbox is read-only"));
        }
        self.target()
    }
}
