//! Serveurs d'un compte : imposés par la politique réseau, saisis, ou ceux du provider.

use mira_vocabulary::{EmailAddress, Provider, ServerSettings};

use super::network_policy::NetworkPolicy;

/// Serveurs à contacter, et serveurs à mémoriser dans le compte (rien si ce sont ceux du provider).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedServers {
    pub connect: ServerSettings,
    pub stored: Option<ServerSettings>,
}

/// Résout les serveurs : politique réseau, puis saisie de l'utilisateur, puis provider
pub fn resolve_servers(
    policy: &NetworkPolicy,
    provider: &dyn Provider,
    address: &EmailAddress,
    requested: Option<ServerSettings>,
) -> ResolvedServers {
    let chosen = policy
        .server_override(address.domain())
        .cloned()
        .or(requested);
    ResolvedServers {
        connect: chosen
            .clone()
            .unwrap_or_else(|| provider.servers_for(address)),
        stored: chosen,
    }
}
