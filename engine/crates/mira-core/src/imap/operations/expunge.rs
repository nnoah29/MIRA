//! Suppression définitive de messages précis (UID EXPUNGE).

use mira_vocabulary::MiraError;

use crate::imap::client::ImapClient;
use crate::imap::errors::status::Operation;
use crate::imap::protocol::quoting::uid_sets;

impl ImapClient {
    /// Supprime définitivement les messages désignés, à condition qu'ils portent `\Deleted` (UIDPLUS).
    pub async fn expunge_uids(&mut self, uids: &[u32]) -> Result<(), MiraError> {
        self.require_selected()?;
        self.require_capability("UIDPLUS")?;
        for set in uid_sets(uids) {
            self.run(&format!("UID EXPUNGE {set}"), Operation::Plain)
                .await?;
        }
        Ok(())
    }
}
