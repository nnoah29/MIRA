//! Accès réseau d'un compte pour les commandes de lecture explicites : serveurs et identifiants du trousseau.

use std::sync::Arc;

use mira_vocabulary::{Account, ErrorCategory, MiraError, ServerSettings};

use crate::accounts::{imap_login_of, resolve_servers};
use crate::api::EngineConfig;
use crate::auth::CredentialVault;
use crate::auth::renewal::renew_access_token;
use crate::imap::ImapClient;
use crate::registry::resolve_account_provider;
use crate::storage::Database;
use crate::sync::{ConnectionBudgets, LeasedClient};

/// Ce qu'il faut pour ouvrir une session IMAP au nom d'un compte du profil.
#[derive(Debug, Clone)]
pub struct ImapAccess {
    config: EngineConfig,
    database: Arc<Database>,
    budgets: ConnectionBudgets,
}

impl ImapAccess {
    /// Assemble l'accès à partir de la configuration du moteur, de sa base et des budgets de connexions
    pub fn new(config: EngineConfig, database: Arc<Database>, budgets: ConnectionBudgets) -> Self {
        Self {
            config,
            database,
            budgets,
        }
    }

    /// Ouvre une session IMAP authentifiée pour le compte, dans le budget de connexions du compte
    pub async fn connect(&self, account: &Account) -> Result<LeasedClient, MiraError> {
        let provider = resolve_account_provider(&self.config.registry, &account.provider)?;
        let limit = provider.descriptor().limits.max_connections;
        let budget = self.budgets.budget_for(account.id.as_str(), limit);
        let permit = budget.acquire_explicit().await?;
        let servers = resolve_servers(
            &self.config.network,
            provider.as_ref(),
            &account.address,
            account.servers.clone(),
        );
        let client = self.connect_renewing(account, &servers.connect).await?;
        Ok(LeasedClient::new(client, permit))
    }

    /// Rafraîchit un jeton expiré avant de se connecter, ou une fois après un refus du serveur
    async fn connect_renewing(
        &self,
        account: &Account,
        servers: &ServerSettings,
    ) -> Result<ImapClient, MiraError> {
        self.renew(account, false).await?;
        match self.connect_once(account, servers).await {
            Err(error) if error.category == ErrorCategory::Authentication => {
                self.renew(account, true).await?;
                self.connect_once(account, servers).await
            }
            other => other,
        }
    }

    /// Rafraîchit le jeton OAuth du compte, toujours ou seulement s'il est expiré
    async fn renew(&self, account: &Account, always: bool) -> Result<(), MiraError> {
        renew_access_token(&self.config, &self.database, &account.id, always).await
    }

    /// Une tentative de connexion IMAP avec le secret actuel du trousseau
    async fn connect_once(
        &self,
        account: &Account,
        servers: &ServerSettings,
    ) -> Result<ImapClient, MiraError> {
        let vault =
            CredentialVault::new(Arc::clone(&self.config.secrets), Arc::clone(&self.database));
        let user = account.address.as_str();
        let secret = vault.load_login(&self.config.profile, &account.id, account.auth, user)?;
        ImapClient::connect(&servers.imap, &imap_login_of(secret)).await
    }
}
