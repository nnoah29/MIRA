//! Rafraîchissement silencieux du jeton d'accès à partir du jeton rangé dans le coffre (7.7).

use mira_vocabulary::error::codes::authentication;
use mira_vocabulary::{
    AccountId, ErrorCategory, MiraError, ProfileId, SecretKind, SuggestedAction,
};

use super::credential_vault::CredentialVault;
use crate::auth::tokens::refresh_tokens;
use crate::ports::{Clock, SecretValue};

impl CredentialVault {
    /// Rafraîchit et range les jetons ; `token_revoked` si le compte doit être réauthentifié (S10)
    pub async fn refresh_access_token(
        &self,
        profile: &ProfileId,
        account: &AccountId,
        token_url: &str,
        client_id: &str,
        clock: &dyn Clock,
    ) -> Result<SecretValue, MiraError> {
        let refresh = self
            .load_secret(profile, account, SecretKind::RefreshToken)?
            .ok_or_else(|| {
                MiraError::new(
                    ErrorCategory::Authentication,
                    authentication::TOKEN_REVOKED,
                    "aucun jeton de rafraîchissement disponible",
                )
                .with_action(SuggestedAction::Reauthenticate)
            })?;
        let tokens = refresh_tokens(token_url, client_id, &refresh, clock).await?;
        self.store_tokens(profile, account, &tokens)?;
        Ok(tokens.access_token)
    }
}
