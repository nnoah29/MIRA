//! Ajout et retrait de drapeaux sur des messages.

use mira_vocabulary::MiraError;

use crate::imap::client::ImapClient;
use crate::imap::errors::status::Operation;
use crate::imap::protocol::quoting::uid_sets;
use crate::imap::types::ImapFlag;

/// Liste de drapeaux entre parenthèses, au format IMAP.
fn flag_list(flags: &[ImapFlag]) -> String {
    let names: Vec<&str> = flags.iter().map(ImapFlag::as_str).collect();
    format!("({})", names.join(" "))
}

impl ImapClient {
    /// Applique `+FLAGS` ou `-FLAGS` à chaque lot d'UID.
    async fn store_flags(
        &mut self,
        sign: char,
        uids: &[u32],
        flags: &[ImapFlag],
    ) -> Result<(), MiraError> {
        self.require_selected()?;
        if flags.is_empty() {
            return Ok(());
        }
        for set in uid_sets(uids) {
            let line = format!("UID STORE {set} {sign}FLAGS.SILENT {}", flag_list(flags));
            self.run(&line, Operation::Plain).await?;
        }
        Ok(())
    }

    /// Ajoute des drapeaux aux messages.
    pub async fn add_flags(&mut self, uids: &[u32], flags: &[ImapFlag]) -> Result<(), MiraError> {
        self.store_flags('+', uids, flags).await
    }

    /// Retire des drapeaux des messages.
    pub async fn remove_flags(
        &mut self,
        uids: &[u32],
        flags: &[ImapFlag],
    ) -> Result<(), MiraError> {
        self.store_flags('-', uids, flags).await
    }
}
