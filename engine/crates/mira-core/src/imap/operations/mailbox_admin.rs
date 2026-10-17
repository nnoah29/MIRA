//! Création, renommage et suppression de boîtes.

use mira_vocabulary::MiraError;

use crate::imap::client::ImapClient;
use crate::imap::errors::status::Operation;
use crate::imap::protocol::quoting::quote;

impl ImapClient {
    /// Crée une boîte.
    pub async fn create_mailbox(&mut self, name: &str) -> Result<(), MiraError> {
        let line = format!("CREATE {}", quote(name)?);
        self.run(&line, Operation::Plain).await.map(|_| ())
    }

    /// Renomme une boîte (et ses sous-boîtes).
    pub async fn rename_mailbox(&mut self, from: &str, to: &str) -> Result<(), MiraError> {
        let line = format!("RENAME {} {}", quote(from)?, quote(to)?);
        self.run(&line, Operation::Plain).await.map(|_| ())
    }

    /// Supprime une boîte.
    pub async fn delete_mailbox(&mut self, name: &str) -> Result<(), MiraError> {
        let line = format!("DELETE {}", quote(name)?);
        self.run(&line, Operation::Plain).await.map(|_| ())
    }
}
