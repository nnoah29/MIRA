//! Récupération de la source complète d'un message, sans le marquer comme lu.

use mira_vocabulary::MiraError;

use crate::imap::client::ImapClient;
use crate::imap::errors::status::Operation;
use crate::imap::parse::fetch::parse_fetch;
use crate::imap::parse::fetched::source_of;

impl ImapClient {
    /// Source brute du message (`BODY.PEEK[]`) ; `None` si le message n'existe plus.
    pub async fn fetch_source(&mut self, uid: u32) -> Result<Option<Vec<u8>>, MiraError> {
        self.require_selected()?;
        let line = format!("UID FETCH {uid} (UID BODY.PEEK[])");
        let response = self.run(&line, Operation::Plain).await?;
        let found = response
            .untagged
            .iter()
            .filter_map(|l| parse_fetch(l))
            .find(|data| data.uid() == Some(uid));
        Ok(found.and_then(source_of))
    }
}
