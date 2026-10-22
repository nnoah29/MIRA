//! Renommage d'un libellé local.

use mira_vocabulary::{LocalLabelId, MiraError};

use super::checks::clean_name;
use super::writing::{announce, single};
use crate::api::contract::Response;
use crate::api::engine::Engine;
use crate::storage::repositories::local_labels::require_label;

/// Renomme le libellé ; nom invalide ou déjà pris : refusé sans écrire ; inconnu : `not_found.local_label`
pub(crate) fn handle_rename_local_label(
    engine: &Engine,
    label: &LocalLabelId,
    name: &str,
) -> Result<Response, MiraError> {
    let name = clean_name(name)?;
    let renamed = engine.database.write(|tx| {
        tx.rename_local_label(label, &name)?;
        require_label(tx, label)
    })?;
    announce(engine)?;
    Ok(single(renamed))
}
