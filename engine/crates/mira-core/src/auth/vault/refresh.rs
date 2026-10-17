//! Rafraîchissement silencieux du jeton d'accès à partir du jeton rangé dans le coffre (7.7).

use mira_vocabulary::error::codes::authentication;
use mira_vocabulary::{
    AccountId, ErrorCategory, MiraError, ProfileId, SecretKind, SuggestedAction,
};

use super::credential_vault::CredentialVault;
use super::refresh_lock::refresh_lock;
use crate::auth::tokens::refresh_tokens_with_secret;
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
        self.refresh_access_token_with_secret(profile, account, token_url, (client_id, None), clock)
            .await
    }

    /// Même rafraîchissement pour un client qui présente aussi un secret d'application
    pub async fn refresh_access_token_with_secret(
        &self,
        profile: &ProfileId,
        account: &AccountId,
        token_url: &str,
        client: (&str, Option<&SecretValue>),
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
        let tokens = refresh_tokens_with_secret(token_url, client, &refresh, clock).await?;
        self.store_tokens(profile, account, &tokens)?;
        Ok(tokens.access_token)
    }

    /// Rafraîchit sous verrou de compte, sauf si un autre appelant a déjà remplacé le jeton `stale`
    pub async fn refresh_access_token_if_stale(
        &self,
        profile: &ProfileId,
        account: &AccountId,
        token_url: &str,
        client_id: &str,
        clock: &dyn Clock,
        stale: Option<&SecretValue>,
    ) -> Result<SecretValue, MiraError> {
        self.refresh_access_token_if_stale_with_secret(
            profile,
            account,
            token_url,
            (client_id, None),
            (clock, stale),
        )
        .await
    }

    /// Même rafraîchissement sous verrou, avec le secret d'application ; `seen` = horloge et jeton périmé
    pub async fn refresh_access_token_if_stale_with_secret(
        &self,
        profile: &ProfileId,
        account: &AccountId,
        token_url: &str,
        client: (&str, Option<&SecretValue>),
        seen: (&dyn Clock, Option<&SecretValue>),
    ) -> Result<SecretValue, MiraError> {
        let lock = refresh_lock(profile, account);
        let _guard = lock.lock().await;
        let current = self.load_secret(profile, account, SecretKind::AccessToken)?;
        if let Some(current) = current
            && seen.1.map(SecretValue::expose) != Some(current.expose())
        {
            return Ok(current);
        }
        self.refresh_access_token_with_secret(profile, account, token_url, client, seen.0)
            .await
    }
}
