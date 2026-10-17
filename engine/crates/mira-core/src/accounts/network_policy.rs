//! Politique réseau injectée : redirection d'un domaine vers d'autres serveurs (tests, relais).

use std::collections::BTreeMap;

use mira_vocabulary::ServerSettings;

use crate::sync::BackgroundSync;

/// Serveurs imposés par domaine d'adresse (vide en production) et mode de la synchronisation en arrière-plan.
#[derive(Debug, Clone, Default)]
pub struct NetworkPolicy {
    pub server_overrides: BTreeMap<String, ServerSettings>,
    pub background_sync: BackgroundSync,
}

impl NetworkPolicy {
    /// Donne les serveurs imposés pour un domaine, s'il y en a
    pub fn server_override(&self, domain: &str) -> Option<&ServerSettings> {
        self.server_overrides.get(domain)
    }
}
