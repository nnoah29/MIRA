//! Accès réseau d'un compte pour les commandes de lecture explicites : serveurs et identifiants du trousseau.

use mira_vocabulary::{Account, MiraError, ProfileId};

use crate::accounts::{NetworkPolicy, imap_login_of, resolve_servers};
use crate::auth::CredentialVault;
use crate::imap::ImapClient;
use crate::registry::{ProviderRegistry, resolve_account_provider};

/// Ce qu'il faut pour ouvrir une session IMAP au nom d'un compte du profil.
#[derive(Debug, Clone)]
pub struct ImapAccess {
    profile: ProfileId,
    vault: CredentialVault,
    network: NetworkPolicy,
    registry: ProviderRegistry,
}

impl ImapAccess {
    /// Assemble l'accès à partir du profil, du coffre, de la politique réseau et du registre
    pub fn new(
        profile: ProfileId,
        vault: CredentialVault,
        network: NetworkPolicy,
        registry: ProviderRegistry,
    ) -> Self {
        Self {
            profile,
            vault,
            network,
            registry,
        }
    }

    /// Ouvre une session IMAP authentifiée pour le compte
    pub async fn connect(&self, account: &Account) -> Result<ImapClient, MiraError> {
        let provider = resolve_account_provider(&self.registry, &account.provider)?;
        let servers = resolve_servers(
            &self.network,
            provider.as_ref(),
            &account.address,
            account.servers.clone(),
        );
        let secret = self.vault.load_login(
            &self.profile,
            &account.id,
            account.auth,
            account.address.as_str(),
        )?;
        ImapClient::connect(&servers.connect.imap, &imap_login_of(secret)).await
    }
}
