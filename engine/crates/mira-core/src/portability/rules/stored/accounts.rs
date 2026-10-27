//! Comptes retrouvés par adresse normalisée.

use mira_vocabulary::{AccountId, EmailAddress};

use super::StoredRefs;
use crate::portability::rules::AccountRefs;

impl AccountRefs for StoredRefs<'_, '_> {
    /// Adresse normalisée du compte
    fn account_address(&self, id: &AccountId) -> Option<String> {
        let account = self.absorb(self.tx.get_account(id))?;
        Some(account.address.as_str().to_string())
    }

    /// Compte dont l'adresse, une fois normalisée, est celle du fichier
    fn account_by_address(&self, address: &str) -> Option<AccountId> {
        let wanted = EmailAddress::parse(address).ok()?;
        let accounts = self.absorb(self.tx.list_accounts().map(Some))?;
        accounts
            .into_iter()
            .find(|account| account.address == wanted)
            .map(|account| account.id)
    }
}
