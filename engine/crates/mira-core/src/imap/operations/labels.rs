//! Libellés propres au service (extension `X-GM-EXT-1`), utilisés seulement si le provider en déclare.

use mira_vocabulary::MiraError;

use crate::imap::client::ImapClient;
use crate::imap::errors::status::Operation;
use crate::imap::protocol::quoting::{quote, uid_sets};

const LABEL_EXTENSION: &str = "X-GM-EXT-1";

impl ImapClient {
    /// Applique `+X-GM-LABELS` ou `-X-GM-LABELS` à chaque lot d'UID.
    async fn store_labels(
        &mut self,
        sign: char,
        uids: &[u32],
        labels: &[String],
    ) -> Result<(), MiraError> {
        self.require_selected()?;
        self.require_capability(LABEL_EXTENSION)?;
        let quoted: Result<Vec<String>, MiraError> = labels.iter().map(|l| quote(l)).collect();
        let list = quoted?.join(" ");
        for set in uid_sets(uids) {
            let line = format!("UID STORE {set} {sign}X-GM-LABELS.SILENT ({list})");
            self.run(&line, Operation::Extension).await?;
        }
        Ok(())
    }

    /// Ajoute des libellés aux messages.
    pub async fn add_labels(&mut self, uids: &[u32], labels: &[String]) -> Result<(), MiraError> {
        self.store_labels('+', uids, labels).await
    }

    /// Retire des libellés des messages.
    pub async fn remove_labels(
        &mut self,
        uids: &[u32],
        labels: &[String],
    ) -> Result<(), MiraError> {
        self.store_labels('-', uids, labels).await
    }
}
