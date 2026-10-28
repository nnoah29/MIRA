//! Idempotence des écritures de règles (invariant 18) : une clé rejouée ne produit pas un second effet.

use mira_vocabulary::error::codes::{internal, invalid_input};
use mira_vocabulary::{ErrorCategory, IdempotencyKey, MiraError, RuleId};
use serde::Serialize;

use crate::storage::Tx;
use crate::storage::repositories::rules::RuleIdempotencyEntry;

/// Résultat de la recherche d'une clé : commande neuve, ou déjà exécutée sur cette règle.
#[derive(Debug)]
pub(super) enum Replay {
    Fresh,
    Replayed { rule: Option<RuleId> },
}

/// Commande d'écriture identifiée par sa clé, son opération et l'empreinte JSON de ses données.
#[derive(Debug)]
pub(super) struct WriteCommand {
    pub(super) key: IdempotencyKey,
    operation: &'static str,
    fingerprint: String,
}

impl WriteCommand {
    /// Prépare la commande ; l'empreinte est la forme JSON des données envoyées par le client
    pub(super) fn new(
        key: IdempotencyKey,
        operation: &'static str,
        payload: &impl Serialize,
    ) -> Result<Self, MiraError> {
        let fingerprint = serde_json::to_string(payload).map_err(|_| {
            MiraError::new(
                ErrorCategory::Internal,
                internal::BUG,
                "commande de règle non sérialisable",
            )
        })?;
        Ok(Self {
            key,
            operation,
            fingerprint,
        })
    }

    /// Cherche la clé ; une autre commande déjà exécutée sous la même clé est un conflit
    pub(super) fn replay(&self, tx: &Tx<'_>) -> Result<Replay, MiraError> {
        let Some(entry) = tx.rule_idempotency_entry(&self.key)? else {
            return Ok(Replay::Fresh);
        };
        if entry.operation != self.operation || entry.fingerprint != self.fingerprint {
            return Err(MiraError::new(
                ErrorCategory::InvalidInput,
                invalid_input::IDEMPOTENCY_CONFLICT,
                "clé d'idempotence déjà utilisée pour une autre commande",
            ));
        }
        Ok(Replay::Replayed { rule: entry.rule })
    }

    /// Mémorise la commande exécutée, dans la même transaction que son effet
    pub(super) fn record(&self, tx: &Tx<'_>, rule: Option<&RuleId>) -> Result<(), MiraError> {
        let entry = RuleIdempotencyEntry {
            operation: self.operation.to_string(),
            fingerprint: self.fingerprint.clone(),
            rule: rule.cloned(),
        };
        tx.record_rule_idempotency(&self.key, &entry)
    }
}
