//! Boîtes retrouvées par adresse de compte et nom serveur.

use mira_vocabulary::MailboxId;

use super::StoredRefs;
use crate::portability::rules::{AccountRefs, MailboxRefs};

impl MailboxRefs for StoredRefs<'_, '_> {
    /// Adresse du compte et nom serveur de la boîte
    fn mailbox_ref(&self, id: &MailboxId) -> Option<(String, String)> {
        let mailbox = self.absorb(self.tx.get_mailbox(id))?;
        let address = self.account_address(&mailbox.account)?;
        Some((address, mailbox.name))
    }

    /// Boîte du compte portant ce nom serveur
    fn mailbox_by_ref(&self, address: &str, name: &str) -> Option<MailboxId> {
        let account = self.account_by_address(address)?;
        let mailbox = self.absorb(self.tx.find_mailbox_by_name(&account, name))?;
        Some(mailbox.id)
    }
}
