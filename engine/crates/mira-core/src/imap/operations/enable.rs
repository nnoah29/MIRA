//! Activation de QRESYNC (RFC 7162) pour une resynchronisation rapide.

use mira_vocabulary::MiraError;

use crate::imap::client::ImapClient;
use crate::imap::errors::status::Operation;

impl ImapClient {
    /// Active QRESYNC si le serveur le permet ; rend `false` si l'extension n'est pas disponible.
    pub async fn enable_qresync(&mut self) -> Result<bool, MiraError> {
        if !self.has_capability("QRESYNC") || !self.has_capability("ENABLE") {
            return Ok(false);
        }
        let response = self.run("ENABLE QRESYNC", Operation::Extension).await?;
        let confirmed = response.untagged.iter().any(|line| {
            let text = String::from_utf8_lossy(line).to_ascii_uppercase();
            text.starts_with("ENABLED") && text.contains("QRESYNC")
        });
        self.qresync = confirmed;
        Ok(confirmed)
    }
}
