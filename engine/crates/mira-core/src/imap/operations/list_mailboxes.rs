//! Liste des boîtes du compte.

use mira_vocabulary::{MiraError, RawMailbox};

use crate::imap::client::ImapClient;
use crate::imap::errors::status::Operation;
use crate::imap::parse::list::parse_list;

impl ImapClient {
    /// Liste toutes les boîtes avec leur séparateur et leurs attributs bruts.
    pub async fn list_mailboxes(&mut self) -> Result<Vec<RawMailbox>, MiraError> {
        let response = self.run("LIST \"\" \"*\"", Operation::Plain).await?;
        Ok(response
            .untagged
            .iter()
            .filter_map(|line| parse_list(line))
            .collect())
    }
}
