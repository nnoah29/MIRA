//! Résultats des recherches serveur terminées, gardés en mémoire en nombre borné.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use mira_vocabulary::error::codes::not_found;
use mira_vocabulary::{ErrorCategory, MailboxId, MessageId, MiraError, OperationId};
use serde::{Deserialize, Serialize};

const MAX_KEPT_RESULTS: usize = 16;

/// Message trouvé par le serveur ; `message` est renseigné s'il est déjà dans le miroir local.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServerSearchHit {
    pub mailbox: MailboxId,
    pub uid: u32,
    pub message: Option<MessageId>,
}

/// Résultat borné d'une recherche serveur ; `truncated` signale que d'autres messages existent.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServerSearchResult {
    pub hits: Vec<ServerSearchHit>,
    pub truncated: bool,
}

/// Mémoire des derniers résultats, indexés par l'opération qui les a produits.
#[derive(Debug, Clone, Default)]
pub struct ServerSearchResults {
    kept: Arc<Mutex<VecDeque<(OperationId, ServerSearchResult)>>>,
}

impl ServerSearchResults {
    /// Garde le résultat d'une opération, en oubliant le plus ancien au-delà de la borne
    pub fn keep(&self, operation: OperationId, result: ServerSearchResult) {
        let mut kept = self.kept.lock().unwrap_or_else(|p| p.into_inner());
        kept.push_back((operation, result));
        while kept.len() > MAX_KEPT_RESULTS {
            kept.pop_front();
        }
    }

    /// Relit le résultat d'une opération terminée, ou `not_found.operation`
    pub fn get(&self, operation: &OperationId) -> Result<ServerSearchResult, MiraError> {
        let kept = self.kept.lock().unwrap_or_else(|p| p.into_inner());
        kept.iter()
            .find(|(id, _)| id == operation)
            .map(|(_, result)| result.clone())
            .ok_or_else(|| {
                MiraError::new(
                    ErrorCategory::NotFound,
                    not_found::OPERATION,
                    "aucun résultat de recherche serveur pour cette opération",
                )
            })
    }
}
