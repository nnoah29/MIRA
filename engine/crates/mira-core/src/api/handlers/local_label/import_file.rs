//! Import des libellés d'un fichier YAML : une seule transaction, un statut par libellé, rejeu par clé.

use std::collections::HashSet;

use mira_vocabulary::{
    IdempotencyKey, LabelImportEntry, LabelImportOutcome, LabelImportReport, MiraError,
};
use serde_json::Value;

use super::idempotency::LabelCommand;
use super::import_file_one::{import_label, shown_name};
use super::writing::announce;
use crate::api::contract::{LocalLabelResponse, Response};
use crate::api::engine::Engine;
use crate::portability::digest::digest_of;
use crate::portability::envelope::unwrap;
use crate::ports::IdGenerator;
use crate::storage::Tx;

/// Importe les libellés du fichier ; la même clé rejouée rend le même rapport sans rien écrire
pub(crate) fn handle_import_labels_file(
    engine: &Engine,
    yaml: String,
    replace_existing: bool,
    key: IdempotencyKey,
) -> Result<Response, MiraError> {
    let items = unwrap(&yaml, "local_labels", "labels")?;
    let payload = (replace_existing, digest_of(&yaml));
    let command = LabelCommand::new(key, "import_labels_file", &payload)?;
    let (report, written) = engine.database.write(|tx| {
        if let Some(report) = command.replay::<LabelImportReport>(tx)? {
            return Ok((report, false));
        }
        let ids = &*engine.config.ids;
        let (report, written) = import_all(tx, ids, items, replace_existing)?;
        command.record(tx, &report)?;
        Ok((report, written))
    })?;
    if written {
        announce(engine)?;
    }
    Ok(Response::LocalLabel(LocalLabelResponse::LabelsFileImport(
        report,
    )))
}

/// Traite les libellés dans l'ordre du fichier ; rend le rapport et s'il y a eu une écriture
fn import_all(
    tx: &Tx<'_>,
    ids: &dyn IdGenerator,
    items: Vec<Value>,
    replace_existing: bool,
) -> Result<(LabelImportReport, bool), MiraError> {
    let mut count = tx.list_local_labels()?.len();
    let (mut entries, mut seen, mut written) = (Vec::new(), HashSet::new(), false);
    for (index, value) in items.into_iter().enumerate() {
        let name = shown_name(&value);
        let outcome = import_label(tx, ids, value, replace_existing, (&mut seen, &mut count))?;
        written |= matches!(
            outcome,
            LabelImportOutcome::Imported | LabelImportOutcome::Replaced
        );
        let position = u32::try_from(index + 1).unwrap_or(u32::MAX);
        entries.push(LabelImportEntry {
            position,
            name,
            outcome,
        });
    }
    Ok((LabelImportReport { entries }, written))
}
