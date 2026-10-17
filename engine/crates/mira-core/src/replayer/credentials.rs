//! Identifiants du rejeu en production : lus dans le trousseau, jeton OAuth rafraîchi en silence (7.7, S10).

use std::sync::Arc;

use mira_vocabulary::{AccountId, MiraError};

use super::shared::ReplayShared;
use crate::accounts::require_account;
use crate::actions::{BoxFuture, ReplayCredentials};
use crate::auth::CredentialVault;
use crate::auth::renewal::renew_access_token;
use crate::smtp::SmtpLogin;

/// Fournisseur d'identifiants d'un moteur, au-dessus du coffre du profil.
#[derive(Debug, Clone)]
pub(crate) struct ProductionCredentials {
    shared: Arc<ReplayShared>,
}

impl ProductionCredentials {
    /// Crée le fournisseur d'identifiants à partir de l'état partagé du moteur
    pub(crate) fn new(shared: &Arc<ReplayShared>) -> Self {
        Self {
            shared: Arc::clone(shared),
        }
    }

    /// Coffre d'identifiants du profil
    pub(crate) fn vault(&self) -> CredentialVault {
        let shared = &self.shared;
        CredentialVault::new(
            Arc::clone(&shared.config.secrets),
            Arc::clone(&shared.database),
        )
    }

    /// Rafraîchit le jeton OAuth du compte, toujours ou seulement s'il est expiré
    async fn renew(&self, account: &AccountId, always: bool) -> Result<(), MiraError> {
        let shared = &self.shared;
        renew_access_token(&shared.config, &shared.database, account, always).await
    }
}

impl ReplayCredentials for ProductionCredentials {
    /// Identifiants SMTP du compte : XOAUTH2 ou mot de passe, selon sa méthode
    fn smtp_login(&self, account: &AccountId) -> Result<SmtpLogin, MiraError> {
        let found = self
            .shared
            .database
            .read(|tx| require_account(tx, account))?;
        let user = found.address.as_str();
        let secret = self
            .vault()
            .load_login(&found.profile, account, found.auth, user)?;
        Ok(secret.into_smtp_login())
    }

    /// Rafraîchit le jeton d'accès s'il est expiré
    fn renew_if_expired<'a>(
        &'a self,
        account: &'a AccountId,
    ) -> BoxFuture<'a, Result<(), MiraError>> {
        Box::pin(async move { self.renew(account, false).await })
    }

    /// Rafraîchit le jeton d'accès sans condition
    fn renew_now<'a>(&'a self, account: &'a AccountId) -> BoxFuture<'a, Result<(), MiraError>> {
        Box::pin(async move { self.renew(account, true).await })
    }
}
