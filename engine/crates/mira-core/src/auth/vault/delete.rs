//! Suppression de tous les secrets d'un compte (suppression du compte, réauthentification).

use mira_vocabulary::{AccountId, MiraError, ProfileId, SecretKind};

use super::credential_vault::CredentialVault;

impl CredentialVault {
    /// Supprime mot de passe, jeton d'accès, jeton de rafraîchissement et expiration
    pub fn delete_all(&self, profile: &ProfileId, account: &AccountId) -> Result<(), MiraError> {
        for kind in [
            SecretKind::Password,
            SecretKind::AccessToken,
            SecretKind::RefreshToken,
        ] {
            self.secrets.delete(&Self::key(profile, account, kind))?;
        }
        self.database.write(|tx| tx.delete_token_expiry(account))
    }
}
