//! Expiration du jeton d'accès, avec une marge de sécurité avant l'échéance réelle.

use mira_vocabulary::{AccountId, MiraError, Timestamp};

use super::credential_vault::CredentialVault;

const SAFETY_MARGIN_MILLIS: i64 = 60_000;

impl CredentialVault {
    /// Dit si le jeton d'accès est expiré (ou le sera dans la minute) ; inconnu = non expiré
    pub fn access_token_expired(
        &self,
        account: &AccountId,
        now: Timestamp,
    ) -> Result<bool, MiraError> {
        let expiry = self.database.read(|tx| tx.token_expiry(account))?;
        Ok(expiry.is_some_and(|at| at <= now.plus_millis(SAFETY_MARGIN_MILLIS)))
    }
}
