//! Clés d'idempotence des écritures sur les modèles de messages : opération, empreinte et résultat rendu.

use mira_vocabulary::{IdempotencyKey, MiraError};
use rusqlite::{OptionalExtension, params};

use crate::storage::Tx;
use crate::storage::errors::storage_error;

/// Commande déjà exécutée sous une clé : son opération, son empreinte et son résultat en JSON.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateIdempotencyEntry {
    pub operation: String,
    pub fingerprint: String,
    pub result: String,
}

impl Tx<'_> {
    /// Retrouve la commande déjà exécutée sous cette clé, s'il y en a une
    pub fn template_idempotency_entry(
        &self,
        key: &IdempotencyKey,
    ) -> Result<Option<TemplateIdempotencyEntry>, MiraError> {
        self.sql()
            .query_row(
                "SELECT operation, fingerprint, result FROM template_idempotency WHERE key = ?1",
                [key.0.as_str()],
                |row| {
                    Ok(TemplateIdempotencyEntry {
                        operation: row.get(0)?,
                        fingerprint: row.get(1)?,
                        result: row.get(2)?,
                    })
                },
            )
            .optional()
            .map_err(storage_error)
    }

    /// Mémorise la commande exécutée sous cette clé
    pub fn record_template_idempotency(
        &self,
        key: &IdempotencyKey,
        entry: &TemplateIdempotencyEntry,
    ) -> Result<(), MiraError> {
        self.sql()
            .execute(
                "INSERT INTO template_idempotency (key, operation, fingerprint, result) VALUES (?1, ?2, ?3, ?4)",
                params![key.0, entry.operation, entry.fingerprint, entry.result],
            )
            .map_err(storage_error)?;
        Ok(())
    }
}
