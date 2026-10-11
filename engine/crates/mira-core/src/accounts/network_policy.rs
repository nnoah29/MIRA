//! Politique réseau injectée : redirection d'un domaine vers d'autres serveurs (tests, relais).

use std::collections::BTreeMap;

use mira_vocabulary::ServerSettings;

/// Serveurs imposés par domaine d'adresse ; vide en production.
#[derive(Debug, Clone, Default)]
pub struct NetworkPolicy {
    pub server_overrides: BTreeMap<String, ServerSettings>,
}

impl NetworkPolicy {
    /// Donne les serveurs imposés pour un domaine, s'il y en a
    pub fn server_override(&self, domain: &str) -> Option<&ServerSettings> {
        self.server_overrides.get(domain)
    }
}
