//! Rapport d'un import de règles mémorisé avec sa clé, pour le rejeu à l'identique.

use mira_vocabulary::error::codes::internal;
use mira_vocabulary::{ErrorCategory, MiraError, RuleImportReport};

use super::idempotency::WriteCommand;
use crate::storage::Tx;

/// Erreur interne : rapport mémorisé illisible
fn unreadable() -> MiraError {
    MiraError::new(
        ErrorCategory::Internal,
        internal::BUG,
        "rapport d'import illisible",
    )
}

impl WriteCommand {
    /// Enregistre la commande puis son rapport, dans la transaction de l'import
    pub(super) fn record_report(
        &self,
        tx: &Tx<'_>,
        report: &RuleImportReport,
    ) -> Result<(), MiraError> {
        self.record(tx, None)?;
        let text = serde_json::to_string(report).map_err(|_| unreadable())?;
        tx.set_rule_import_result(&self.key, &text)
    }

    /// Rapport mémorisé par l'exécution d'origine de cette commande
    pub(super) fn stored_report(&self, tx: &Tx<'_>) -> Result<RuleImportReport, MiraError> {
        let text = tx.rule_import_result(&self.key)?.ok_or_else(unreadable)?;
        serde_json::from_str(&text).map_err(|_| unreadable())
    }
}
