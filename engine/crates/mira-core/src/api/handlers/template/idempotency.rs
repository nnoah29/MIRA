//! Idempotence des écritures de modèles de messages (invariant 18) : une clé rejouée rend le même résultat.

use mira_vocabulary::error::codes::{internal, invalid_input};
use mira_vocabulary::{ErrorCategory, IdempotencyKey, MiraError};
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::storage::Tx;
use crate::storage::repositories::message_templates::TemplateIdempotencyEntry;

/// Commande d'écriture identifiée par sa clé, son opération et l'empreinte JSON de ses données.
#[derive(Debug)]
pub(super) struct TemplateCommand {
    key: IdempotencyKey,
    operation: &'static str,
    fingerprint: String,
}

/// Erreur interne : un résultat ne se sérialise pas ou ne se relit pas
fn unreadable() -> MiraError {
    MiraError::new(
        ErrorCategory::Internal,
        internal::BUG,
        "résultat de modèle illisible",
    )
}

impl TemplateCommand {
    /// Prépare la commande ; l'empreinte est la forme JSON des données envoyées par le client
    pub(super) fn new(
        key: IdempotencyKey,
        operation: &'static str,
        payload: &impl Serialize,
    ) -> Result<Self, MiraError> {
        let fingerprint = serde_json::to_string(payload).map_err(|_| unreadable())?;
        Ok(Self {
            key,
            operation,
            fingerprint,
        })
    }

    /// Résultat de la première exécution si la clé a déjà servi pour cette commande ; autre commande : conflit
    pub(super) fn replay<T: DeserializeOwned>(&self, tx: &Tx<'_>) -> Result<Option<T>, MiraError> {
        let Some(entry) = tx.template_idempotency_entry(&self.key)? else {
            return Ok(None);
        };
        if entry.operation != self.operation || entry.fingerprint != self.fingerprint {
            return Err(MiraError::new(
                ErrorCategory::InvalidInput,
                invalid_input::IDEMPOTENCY_CONFLICT,
                "clé d'idempotence déjà utilisée pour une autre commande",
            ));
        }
        serde_json::from_str(&entry.result)
            .map(Some)
            .map_err(|_| unreadable())
    }

    /// Mémorise le résultat, dans la même transaction que l'effet
    pub(super) fn record(&self, tx: &Tx<'_>, result: &impl Serialize) -> Result<(), MiraError> {
        let entry = TemplateIdempotencyEntry {
            operation: self.operation.to_string(),
            fingerprint: self.fingerprint.clone(),
            result: serde_json::to_string(result).map_err(|_| unreadable())?,
        };
        tx.record_template_idempotency(&self.key, &entry)
    }
}
