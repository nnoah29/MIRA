//! Ligne de journal d'une génération : ouverte avant le réseau, complétée à chaque fin, bornée par la rétention.

use std::sync::Arc;

use mira_vocabulary::error::codes::not_found;
use mira_vocabulary::{AiJournalEntry, AiJournalFinish, MiraError, OperationId, Timestamp};

use super::ending::Ending;
use super::prepare::ChatPlan;
use super::progress::Progress;
use crate::api::Engine;
use crate::storage::{Database, Tx};

const MAX_ROWS: usize = 2000;
const MAX_AGE_MILLIS: i64 = 90 * 86_400_000;

/// Écrit la ligne `running` de la génération puis applique la rétention, avant tout accès réseau.
pub(crate) fn open_row(
    engine: &Engine,
    plan: &ChatPlan,
    operation: &OperationId,
) -> Result<(), MiraError> {
    let at = engine.config.clock.now();
    let entry = AiJournalEntry {
        id: engine.config.ids.next_id(),
        at,
        operation: operation.clone(),
        connection: plan.connection.clone(),
        provider: plan.provider.clone(),
        model: plan.model.clone(),
        local: plan.local,
        kinds: plan.kinds.clone(),
        parts: plan.parts,
        chars: plan.chars,
        masked: 0,
        input_tokens: None,
        output_tokens: None,
        outcome: "running".to_string(),
    };
    engine.database.write(|tx| {
        tx.insert_ai_journal(&entry)?;
        trim(tx, at)
    })
}

/// Complète la ligne de l'opération puis applique la rétention ; un échec est tracé sans contenu.
pub(super) fn close_row(
    database: &Arc<Database>,
    operation: &OperationId,
    finish: &AiJournalFinish,
    now: Timestamp,
) {
    let closed = database.write(|tx| {
        match tx.finish_ai_journal(operation, finish) {
            Err(error) if error.code != not_found::OPERATION.0 => return Err(error),
            _ => {}
        }
        trim(tx, now)
    });
    if let Err(error) = closed {
        tracing::warn!(code = %error.code, "ligne de journal d'IA non complétée");
    }
}

/// Ce que l'on retient d'une génération terminée : jetons agrégés et issue stable.
pub(super) fn finish_of(progress: &Progress, ending: &Ending) -> AiJournalFinish {
    let (input_tokens, output_tokens) = progress.tokens();
    AiJournalFinish {
        masked: 0,
        input_tokens,
        output_tokens,
        outcome: ending.outcome(),
    }
}

/// Marque `interrupted` les lignes restées `running` d'un arrêt brutal ; un échec est tracé sans empêcher l'ouverture.
pub(crate) fn interrupt_running(database: &Arc<Database>) {
    if let Err(error) = database.write(|tx| tx.interrupt_running_ai_journal()) {
        tracing::warn!(code = %error.code, "reprise du journal d'IA non appliquée");
    }
}

/// Garde les 2000 dernières lignes et 90 jours, sans jamais retirer une ligne en cours.
fn trim(tx: &Tx<'_>, now: Timestamp) -> Result<(), MiraError> {
    let oldest = Timestamp::from_millis(now.as_millis().saturating_sub(MAX_AGE_MILLIS));
    tx.trim_ai_journal(MAX_ROWS, oldest).map(|_| ())
}
