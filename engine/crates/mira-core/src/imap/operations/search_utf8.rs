//! Recherche de texte non ASCII : `SEARCH CHARSET UTF-8` avec la requête en littéral.

use mira_vocabulary::MiraError;

use crate::imap::client::ImapClient;
use crate::imap::errors::status::Operation;
use crate::imap::parse::search::collect_uids;
use crate::imap::protocol::quoting::reject_control_characters;

impl ImapClient {
    /// UID des messages dont le texte contient la requête, envoyée en UTF-8 dans un littéral.
    pub(super) async fn search_text_utf8(&mut self, query: &str) -> Result<Vec<u32>, MiraError> {
        self.require_selected()?;
        reject_control_characters(query)?;
        let response = self
            .wire
            .command_with_literal("UID SEARCH CHARSET UTF-8 TEXT", query.as_bytes())
            .await?
            .require_ok(Operation::Plain)?;
        Ok(collect_uids(&response.untagged))
    }
}
