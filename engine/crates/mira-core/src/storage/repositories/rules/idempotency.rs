//! Clés d'idempotence des écritures sur les règles : opération, empreinte de la commande et règle visée.

use mira_vocabulary::{IdempotencyKey, MiraError, RuleId};
use rusqlite::{OptionalExtension, params};

use crate::storage::Tx;
use crate::storage::errors::storage_error;

/// Commande déjà exécutée sous une clé : son opération, son empreinte et la règle concernée.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleIdempotencyEntry {
    pub operation: String,
    pub fingerprint: String,
    pub rule: Option<RuleId>,
}

impl Tx<'_> {
    /// Retrouve la commande déjà exécutée sous cette clé, s'il y en a une
    pub fn rule_idempotency_entry(
        &self,
        key: &IdempotencyKey,
    ) -> Result<Option<RuleIdempotencyEntry>, MiraError> {
        self.sql()
            .query_row(
                "SELECT operation, fingerprint, rule_id FROM rule_idempotency WHERE key = ?1",
                [key.0.as_str()],
                |row| {
                    Ok(RuleIdempotencyEntry {
                        operation: row.get(0)?,
                        fingerprint: row.get(1)?,
                        rule: row.get::<_, Option<String>>(2)?.map(RuleId::from_raw),
                    })
                },
            )
            .optional()
            .map_err(storage_error)
    }

    /// Mémorise la commande exécutée sous cette clé
    pub fn record_rule_idempotency(
        &self,
        key: &IdempotencyKey,
        entry: &RuleIdempotencyEntry,
    ) -> Result<(), MiraError> {
        self.sql()
            .execute(
                "INSERT INTO rule_idempotency (key, operation, fingerprint, rule_id) VALUES (?1, ?2, ?3, ?4)",
                params![
                    key.0,
                    entry.operation,
                    entry.fingerprint,
                    entry.rule.as_ref().map(RuleId::as_str),
                ],
            )
            .map_err(storage_error)?;
        Ok(())
    }
}
