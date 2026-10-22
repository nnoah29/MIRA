//! Contrôles communs aux écritures de libellés locaux : nom, plafond de libellés.

use mira_vocabulary::MiraError;

use crate::actions::errors::input::invalid_parameter;
use crate::storage::Tx;

pub(super) const MAX_NAME_LEN: usize = 64;
pub(super) const MAX_LABELS: usize = 200;
pub(super) const MAX_THREADS_PER_CALL: usize = 1000;
pub(super) const MAX_IMPORTED_KEYS: usize = 50_000;

/// Nom sans espaces de bord, ou refus s'il est vide ou trop long (message fixe, sans le texte saisi)
pub(super) fn clean_name(name: &str) -> Result<String, MiraError> {
    let cleaned = name.trim();
    if cleaned.is_empty() {
        return Err(invalid_parameter("le nom du libellé est vide"));
    }
    if cleaned.len() > MAX_NAME_LEN {
        return Err(invalid_parameter("le nom du libellé est trop long"));
    }
    Ok(cleaned.to_string())
}

/// Nombre de libellés existants, après avoir vérifié qu'on peut en ajouter `extra` sans passer le plafond
pub(super) fn ensure_room(tx: &Tx<'_>, extra: usize) -> Result<usize, MiraError> {
    let count = tx.list_local_labels()?.len();
    if count + extra > MAX_LABELS {
        return Err(invalid_parameter("trop de libellés locaux"));
    }
    Ok(count)
}
