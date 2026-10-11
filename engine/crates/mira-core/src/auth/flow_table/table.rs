//! Table des flux OAuth en cours, protégée par un verrou jamais tenu pendant une attente.

use std::collections::BTreeMap;
use std::sync::{Mutex, MutexGuard};

use mira_vocabulary::error::codes::invalid_input;
use mira_vocabulary::{ErrorCategory, MiraError};

use super::entry::{FlowEntry, FlowSummary};

/// Flux en cours, par identifiant de flux.
#[derive(Debug, Default)]
pub struct OAuthFlows {
    entries: Mutex<BTreeMap<String, FlowEntry>>,
}

impl OAuthFlows {
    /// Crée une table vide
    pub fn new() -> Self {
        Self::default()
    }

    /// Prend le verrou de la table en ignorant un éventuel empoisonnement
    pub(super) fn entries(&self) -> MutexGuard<'_, BTreeMap<String, FlowEntry>> {
        self.entries
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Enregistre un flux sous son identifiant
    pub fn register(&self, flow: String, entry: FlowEntry) {
        self.entries().insert(flow, entry);
    }

    /// Oublie un flux, terminé ou non
    pub fn remove(&self, flow: &str) {
        self.entries().remove(flow);
    }

    /// Décrit un flux qui a déjà obtenu ses jetons, ou échoue s'il est inconnu ou inachevé
    pub fn completed_summary(&self, flow: &str) -> Result<FlowSummary, MiraError> {
        let summary = self.entries().get(flow).map(FlowEntry::summary);
        match summary {
            Some(summary) if summary.completed => Ok(summary),
            _ => Err(unknown_flow()),
        }
    }
}

/// Erreur d'un flux inconnu, déjà consommé ou pas encore terminé
pub(super) fn unknown_flow() -> MiraError {
    MiraError::new(
        ErrorCategory::InvalidInput,
        invalid_input::PARAMETER,
        "flux d'autorisation inconnu, en cours ou déjà utilisé",
    )
}
