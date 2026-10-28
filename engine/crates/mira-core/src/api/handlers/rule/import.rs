//! Import des règles d'un fichier YAML : une seule transaction, un statut par règle, rejeu par clé.

use std::collections::HashSet;

use mira_vocabulary::{IdempotencyKey, MiraError, RuleId, RuleImportEntry, RuleImportReport};
use serde_json::Value;

use super::idempotency::{Replay, WriteCommand};
use super::import_one::import_item;
use super::writing::announce;
use crate::api::contract::{Response, RuleResponse};
use crate::api::engine::Engine;
use crate::portability::digest::digest_of;
use crate::portability::envelope::unwrap;
use crate::portability::rules::StoredRefs;
use crate::storage::Tx;

/// Importe les règles du fichier ; la même clé rejouée rend le même rapport sans rien écrire
pub(crate) fn handle_import_rules(
    engine: &Engine,
    yaml: String,
    replace_existing: bool,
    key: IdempotencyKey,
) -> Result<Response, MiraError> {
    let items = unwrap(&yaml, "rules", "rules")?;
    let command = WriteCommand::new(key, "import_rules", &(replace_existing, digest_of(&yaml)))?;
    let (report, written) = engine.database.write(|tx| {
        if let Replay::Replayed { .. } = command.replay(tx)? {
            return Ok((command.stored_report(tx)?, Vec::new()));
        }
        let (report, written) = import_all(tx, items, replace_existing)?;
        command.record_report(tx, &report)?;
        Ok((report, written))
    })?;
    for id in &written {
        announce(engine, id)?;
    }
    Ok(Response::Rule(RuleResponse::RulesImport(report)))
}

/// Traite les règles dans l'ordre du fichier ; rend le rapport et les identifiants écrits
fn import_all(
    tx: &Tx<'_>,
    items: Vec<Value>,
    replace_existing: bool,
) -> Result<(RuleImportReport, Vec<RuleId>), MiraError> {
    let refs = StoredRefs::new(tx);
    let (mut entries, mut written, mut seen) = (Vec::new(), Vec::new(), HashSet::new());
    for (index, value) in items.into_iter().enumerate() {
        let name = value
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        let (outcome, id) = import_item(tx, &refs, value, replace_existing, &mut seen)?;
        written.extend(id);
        let position = u32::try_from(index + 1).unwrap_or(u32::MAX);
        entries.push(RuleImportEntry {
            position,
            name,
            outcome,
        });
    }
    Ok((RuleImportReport { entries }, written))
}
