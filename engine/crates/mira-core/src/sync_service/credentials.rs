//! Identifiants IMAP de la synchronisation en production : trousseau du profil, jeton OAuth rafraîchi en silence.

use std::sync::Arc;

use mira_vocabulary::error::codes::{authentication, secrets};
use mira_vocabulary::{Account, AuthMethodKind, ErrorCategory, MiraError, SuggestedAction};

use crate::accounts::imap_login_of;
use crate::api::EngineConfig;
use crate::auth::CredentialVault;
use crate::auth::renewal::renew_access_token;
use crate::imap::ImapLogin;
use crate::storage::Database;
use crate::sync::{BoxFuture, CredentialProvider};

/// Fournit à la synchronisation les identifiants des comptes du profil.
#[derive(Debug, Clone)]
pub(crate) struct ProfileCredentials {
    config: EngineConfig,
    database: Arc<Database>,
}

impl ProfileCredentials {
    /// Crée le fournisseur à partir de la configuration du moteur et de sa base
    pub(crate) fn new(config: EngineConfig, database: Arc<Database>) -> Self {
        Self { config, database }
    }

    /// Identifiants IMAP du compte ; un jeton expiré est rafraîchi d'abord
    async fn login(&self, account: &Account) -> Result<ImapLogin, MiraError> {
        renew_access_token(&self.config, &self.database, &account.id, false).await?;
        let vault =
            CredentialVault::new(Arc::clone(&self.config.secrets), Arc::clone(&self.database));
        let user = account.address.as_str();
        let secret = vault
            .load_login(&account.profile, &account.id, account.auth, user)
            .map_err(missing_secret_is_a_rejection)?;
        Ok(imap_login_of(secret))
    }
}

/// Un secret absent du trousseau demande une nouvelle authentification, comme un refus du serveur
fn missing_secret_is_a_rejection(error: MiraError) -> MiraError {
    if error.code != secrets::SECRET_MISSING.0 {
        return error;
    }
    MiraError::new(
        ErrorCategory::Authentication,
        authentication::REJECTED,
        "secret absent du trousseau",
    )
    .with_action(SuggestedAction::Reauthenticate)
}

impl CredentialProvider for ProfileCredentials {
    /// Identifiants IMAP à présenter au serveur
    fn imap_login<'a>(
        &'a self,
        account: &'a Account,
    ) -> BoxFuture<'a, Result<ImapLogin, MiraError>> {
        Box::pin(self.login(account))
    }

    /// Rafraîchit le jeton d'accès d'un compte OAuth ; les autres méthodes n'ont rien à renouveler
    fn refresh<'a>(&'a self, account: &'a Account) -> BoxFuture<'a, Result<bool, MiraError>> {
        Box::pin(async move {
            renew_access_token(&self.config, &self.database, &account.id, true).await?;
            Ok(account.auth == AuthMethodKind::OAuth2)
        })
    }
}
