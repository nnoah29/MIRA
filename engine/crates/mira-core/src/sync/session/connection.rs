//! Connexion IMAP d'un compte : permis du budget, identifiants, capacités effectives, QRESYNC.

use std::collections::BTreeSet;

use mira_vocabulary::{ErrorCategory, MiraError, ServerEndpoint};
use tokio::sync::OwnedSemaphorePermit;

use super::credentials::CredentialProvider;
use crate::accounts::resolve_servers;
use crate::capabilities::{EffectiveCapabilities, ServerCapability, parse_capabilities};
use crate::imap::ImapClient;
use crate::sync::context::SyncContext;

/// Connexion authentifiée qui garde son permis du budget tant qu'elle vit.
#[derive(Debug)]
pub struct Connection {
    pub client: ImapClient,
    pub capabilities: EffectiveCapabilities,
    _permit: OwnedSemaphorePermit,
}

/// Serveur IMAP du compte : politique réseau, puis réglages saisis, puis ceux du provider.
fn imap_endpoint(ctx: &SyncContext) -> ServerEndpoint {
    let requested = ctx.account.servers.clone();
    let provider = ctx.provider.as_ref();
    let servers = resolve_servers(&ctx.network, provider, &ctx.account.address, requested);
    servers.connect.imap
}

/// Se connecte ; un refus d'authentification déclenche un rafraîchissement puis un seul nouvel essai.
async fn authenticated_client(
    ctx: &SyncContext,
    credentials: &dyn CredentialProvider,
) -> Result<ImapClient, MiraError> {
    let endpoint = imap_endpoint(ctx);
    let login = credentials.imap_login(&ctx.account).await?;
    match ImapClient::connect(&endpoint, &login).await {
        Err(error) if error.category == ErrorCategory::Authentication => {
            if !credentials.refresh(&ctx.account).await? {
                return Err(error);
            }
            let renewed = credentials.imap_login(&ctx.account).await?;
            ImapClient::connect(&endpoint, &renewed).await
        }
        other => other,
    }
}

/// Capacités effectives ; QRESYNC est retiré si le serveur refuse de l'activer.
async fn effective_capabilities(
    ctx: &SyncContext,
    client: &mut ImapClient,
) -> Result<EffectiveCapabilities, MiraError> {
    let quirks = ctx.provider.descriptor().quirks;
    let mut announced: BTreeSet<ServerCapability> = parse_capabilities(client.capabilities());
    let effective = EffectiveCapabilities::compute(&announced, &quirks);
    if effective.has(ServerCapability::Qresync) && !client.enable_qresync().await? {
        announced.remove(&ServerCapability::Qresync);
        return Ok(EffectiveCapabilities::compute(&announced, &quirks));
    }
    Ok(effective)
}

/// Ouvre une connexion pour le compte avec un permis déjà obtenu du budget.
pub async fn open_connection(
    ctx: &SyncContext,
    permit: OwnedSemaphorePermit,
    credentials: &dyn CredentialProvider,
) -> Result<Connection, MiraError> {
    let mut client = authenticated_client(ctx, credentials).await?;
    let capabilities = effective_capabilities(ctx, &mut client).await?;
    Ok(Connection {
        client,
        capabilities,
        _permit: permit,
    })
}
