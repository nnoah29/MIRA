//! Ouverture d'une boîte, avec resynchronisation rapide (QRESYNC) si elle est demandée.

use mira_vocabulary::error::codes::server;
use mira_vocabulary::{ErrorCategory, MiraError};

use crate::imap::client::ImapClient;
use crate::imap::errors::status::Operation;
use crate::imap::parse::select::parse_select;
use crate::imap::protocol::quoting::quote_mailbox;
use crate::imap::types::{ResyncFrom, SelectedMailbox};

/// Erreur lorsque QRESYNC est demandé sans avoir été activé.
fn qresync_not_enabled() -> MiraError {
    MiraError::new(
        ErrorCategory::Server,
        server::EXTENSION_MISSING,
        "QRESYNC n'est pas activé sur cette session",
    )
}

impl ImapClient {
    /// Écrit la commande SELECT adaptée aux extensions disponibles.
    fn select_line(&self, mailbox: &str, resync: Option<ResyncFrom>) -> Result<String, MiraError> {
        let name = quote_mailbox(mailbox)?;
        match resync {
            Some(_) if !self.qresync => Err(qresync_not_enabled()),
            Some(from) => Ok(format!(
                "SELECT {name} (QRESYNC ({} {}))",
                from.uid_validity, from.highest_modseq
            )),
            None if self.has_capability("CONDSTORE") => Ok(format!("SELECT {name} (CONDSTORE)")),
            None => Ok(format!("SELECT {name}")),
        }
    }

    /// Ouvre la boîte et rend ses marqueurs ; avec `resync`, rend aussi les changements et disparitions.
    pub async fn select(
        &mut self,
        mailbox: &str,
        resync: Option<ResyncFrom>,
    ) -> Result<SelectedMailbox, MiraError> {
        let line = self.select_line(mailbox, resync)?;
        self.selected = None;
        let operation = if resync.is_some() {
            Operation::Extension
        } else {
            Operation::Plain
        };
        let response = self.run(&line, operation).await?;
        self.selected = Some(mailbox.to_string());
        Ok(parse_select(&response.untagged))
    }
}
