//! Rangement des mots de passe et des jetons dans le trousseau.

use mira_vocabulary::{AccountId, MiraError, ProfileId, SecretKind};

use super::credential_vault::CredentialVault;
use crate::auth::tokens::TokenSet;
use crate::ports::SecretValue;

impl CredentialVault {
    /// Range le mot de passe (ou mot de passe d'application) d'un compte
    pub fn store_password(
        &self,
        profile: &ProfileId,
        account: &AccountId,
        password: &SecretValue,
    ) -> Result<(), MiraError> {
        let key = Self::key(profile, account, SecretKind::Password);
        self.secrets.put(&key, password)
    }

    /// Range les jetons ; le jeton de rafraîchissement n'est remplacé que s'il est fourni
    pub fn store_tokens(
        &self,
        profile: &ProfileId,
        account: &AccountId,
        tokens: &TokenSet,
    ) -> Result<(), MiraError> {
        let access = Self::key(profile, account, SecretKind::AccessToken);
        self.secrets.put(&access, &tokens.access_token)?;
        if let Some(refresh) = &tokens.refresh_token {
            let key = Self::key(profile, account, SecretKind::RefreshToken);
            self.secrets.put(&key, refresh)?;
        }
        self.database
            .write(|tx| tx.set_token_expiry(account, tokens.expires_at))
    }
}
