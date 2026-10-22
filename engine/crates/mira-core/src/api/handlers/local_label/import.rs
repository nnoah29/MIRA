//! Reprise des libellés de l'app : libellés, poses sur les conversations, rapport ; idempotente par clé.

use mira_vocabulary::{IdempotencyKey, MiraError};

use super::checks::{MAX_IMPORTED_KEYS, MAX_LABELS, clean_name};
use super::idempotency::LabelCommand;
use super::import_apply::apply_import;
use super::writing::announce;
use crate::actions::errors::input::invalid_parameter;
use crate::api::contract::{ImportReport, ImportedLabel, LocalLabelResponse, Response};
use crate::api::engine::Engine;

/// Refuse avant toute écriture un nom invalide ou un volume au-delà des plafonds
fn check_import(labels: &[ImportedLabel]) -> Result<(), MiraError> {
    if labels.len() > MAX_LABELS {
        return Err(invalid_parameter("trop de libellés locaux"));
    }
    let keys: usize = labels.iter().map(|l| l.conversations.len()).sum();
    if keys > MAX_IMPORTED_KEYS {
        return Err(invalid_parameter("trop de conversations à reprendre"));
    }
    labels
        .iter()
        .try_for_each(|l| clean_name(&l.name).map(|_| ()))
}

/// Reprend les libellés et leurs poses ; la même clé rejouée rend le même rapport sans rien écrire
pub(crate) fn handle_import_local_labels(
    engine: &Engine,
    labels: &[ImportedLabel],
    key: IdempotencyKey,
) -> Result<Response, MiraError> {
    check_import(labels)?;
    let command = LabelCommand::new(key, "import_local_labels", &labels)?;
    let (report, written) = engine.database.write(|tx| {
        if let Some(report) = command.replay::<ImportReport>(tx)? {
            return Ok((report, false));
        }
        let report = apply_import(tx, &*engine.config.ids, labels)?;
        command.record(tx, &report)?;
        Ok((report, true))
    })?;
    if written && !labels.is_empty() {
        announce(engine)?;
    }
    Ok(Response::LocalLabel(LocalLabelResponse::Imported(report)))
}
