//! Constructeurs du faux provider de test et lecture de ses attributs bruts.

use mira_vocabulary::{ProviderLimits, RawMessageAttributes};

use super::provider::SyncTestProvider;

impl SyncTestProvider {
    /// Service à dossiers, sans identifiant stable
    pub fn folders() -> Self {
        Self {
            stable_ids: false,
            limits: ProviderLimits::default(),
        }
    }

    /// Service à libellés avec identifiant stable et indice de fil
    pub fn with_stable_ids() -> Self {
        Self {
            stable_ids: true,
            limits: ProviderLimits::default(),
        }
    }

    /// Fixe le nombre maximal de connexions
    pub fn max_connections(mut self, max: u32) -> Self {
        self.limits.max_connections = max;
        self
    }
}

/// Valeur brute d'un attribut du serveur
pub fn raw_item(raw: &RawMessageAttributes, name: &str) -> Option<String> {
    let found = raw.items.iter().find(|(key, _)| key == name);
    found.map(|(_, value)| value.clone())
}
