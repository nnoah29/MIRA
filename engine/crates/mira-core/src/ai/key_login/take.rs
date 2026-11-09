//! Étapes d'un flux dans la table : prise en charge pour l'attendre, annulation.

use std::sync::Arc;

use mira_vocabulary::error::codes::invalid_input;
use mira_vocabulary::{AiConnectionId, ErrorCategory, MiraError};
use tokio::sync::Notify;

use super::table::AiKeyLogins;
use crate::auth::PendingFlow;

/// Flux sorti de la table pour être attendu : le flux, son signal d'annulation, la connexion visée
pub type TakenKeyLogin = (PendingFlow, Arc<Notify>, AiConnectionId);

impl AiKeyLogins {
    /// Sort le flux de l'attente ; inconnu, expiré ou déjà pris : `invalid_input.parameter`
    pub fn begin_completion(&self, flow: &str) -> Result<TakenKeyLogin, MiraError> {
        let mut entries = self.entries();
        let entry = entries.get_mut(flow).ok_or_else(unknown_flow)?;
        let pending = entry.pending.take().ok_or_else(unknown_flow)?;
        Ok((pending, Arc::clone(&entry.cancel), entry.connection.clone()))
    }

    /// Annule et oublie le flux ; un flux inconnu n'est pas une erreur
    pub fn cancel(&self, flow: &str) {
        if let Some(entry) = self.entries().remove(flow) {
            entry.cancel.notify_one();
        }
    }
}

/// Erreur d'un flux inconnu, expiré ou déjà pris
fn unknown_flow() -> MiraError {
    MiraError::new(
        ErrorCategory::InvalidInput,
        invalid_input::PARAMETER,
        "connexion par navigateur inconnue, expirée ou déjà utilisée",
    )
}
