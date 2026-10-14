//! Ouverture d'une session IMAP pour le rejeu : serveurs du compte, identifiants du trousseau, jeton rafraîchi.

use std::sync::Arc;

use mira_vocabulary::{Account, MiraError, ProviderQuirks, ServerEndpoint, ServerSettings};

use super::credentials::ProductionCredentials;
use super::shared::ReplayShared;
use crate::accounts::{imap_login_of, resolve_servers};
use crate::actions::ReplayCredentials;
use crate::imap::ImapClient;
use crate::registry::resolve_account_provider;

/// Session ouverte : connexion IMAP, serveur SMTP du compte et particularités du service.
#[derive(Debug)]
pub(crate) struct Session {
    pub(crate) client: ImapClient,
    pub(crate) smtp: ServerEndpoint,
    pub(crate) quirks: ProviderQuirks,
}

/// Se connecte au serveur du compte ; un jeton expiré est rafraîchi avant, ou après un refus
pub(crate) async fn open_session(
    shared: &Arc<ReplayShared>,
    account: &Account,
) -> Result<Session, MiraError> {
    let config = &shared.config;
    let provider = resolve_account_provider(&config.registry, &account.provider)?;
    let servers = resolve_servers(
        &config.network,
        provider.as_ref(),
        &account.address,
        account.servers.clone(),
    )
    .connect;
    let credentials = ProductionCredentials::new(shared);
    credentials.renew_if_expired(&account.id).await?;
    let client = match connect(&credentials, account, &servers).await {
        Err(error) if is_refusal(&error) => {
            credentials.renew_now(&account.id).await?;
            connect(&credentials, account, &servers).await?
        }
        other => other?,
    };
    Ok(Session {
        client,
        smtp: servers.smtp,
        quirks: provider.descriptor().quirks,
    })
}

/// Une tentative de connexion IMAP avec le secret actuel du trousseau
async fn connect(
    credentials: &ProductionCredentials,
    account: &Account,
    servers: &ServerSettings,
) -> Result<ImapClient, MiraError> {
    let user = account.address.as_str();
    let secret =
        credentials
            .vault()
            .load_login(&account.profile, &account.id, account.auth, user)?;
    ImapClient::connect(&servers.imap, &imap_login_of(secret)).await
}

/// Dit si l'erreur est un refus d'authentification
fn is_refusal(error: &MiraError) -> bool {
    error.category == mira_vocabulary::ErrorCategory::Authentication
}
