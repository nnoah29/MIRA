//! Ensemble d'extensions annoncées par un faux serveur.

use std::collections::BTreeSet;

use super::Capability;

/// Ensemble d'extensions IMAP activées sur un faux serveur.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CapabilitySet {
    pub(super) items: BTreeSet<Capability>,
}

impl CapabilitySet {
    /// Renvoie un ensemble auquel l'extension est ajoutée
    pub fn with(mut self, capability: Capability) -> Self {
        self.items.insert(capability);
        self
    }

    /// Renvoie un ensemble dont l'extension est retirée
    pub fn without(mut self, capability: Capability) -> Self {
        self.items.remove(&capability);
        self
    }

    /// Indique si l'extension est activée
    pub fn contains(&self, capability: Capability) -> bool {
        self.items.contains(&capability)
    }

    /// Mots-clés CAPABILITY correspondant à l'ensemble
    pub(crate) fn tokens(&self) -> Vec<&'static str> {
        self.items.iter().map(|item| item.token()).collect()
    }
}
