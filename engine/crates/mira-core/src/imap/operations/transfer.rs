//! Copie et déplacement de messages vers une autre boîte.

use mira_vocabulary::MiraError;

use crate::imap::client::ImapClient;
use crate::imap::errors::status::Operation;
use crate::imap::protocol::quoting::{quote, uid_sets};

impl ImapClient {
    /// Envoie `UID COPY` ou `UID MOVE` pour chaque lot d'UID.
    async fn transfer(&mut self, verb: &str, uids: &[u32], to: &str) -> Result<(), MiraError> {
        self.require_selected()?;
        let target = quote(to)?;
        for set in uid_sets(uids) {
            let line = format!("UID {verb} {set} {target}");
            self.run(&line, Operation::Plain).await?;
        }
        Ok(())
    }

    /// Copie les messages dans une autre boîte.
    pub async fn copy(&mut self, uids: &[u32], to: &str) -> Result<(), MiraError> {
        self.transfer("COPY", uids, to).await
    }

    /// Déplace les messages (extension MOVE obligatoire ; le repli copie puis supprime est décidé ailleurs).
    pub async fn move_messages(&mut self, uids: &[u32], to: &str) -> Result<(), MiraError> {
        self.require_capability("MOVE")?;
        self.transfer("MOVE", uids, to).await
    }
}

