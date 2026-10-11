//! Fermeture propre de la session.

use mira_vocabulary::MiraError;

use crate::imap::client::ImapClient;

impl ImapClient {
    /// Termine la session par LOGOUT ; une connexion déjà coupée n'est pas une erreur.
    pub async fn logout(mut self) -> Result<(), MiraError> {
        let _ = self.wire.command("LOGOUT").await;
        Ok(())
    }
}
