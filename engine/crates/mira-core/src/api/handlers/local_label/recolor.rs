//! Changement de couleur d'un libellé local.

use mira_vocabulary::{LocalLabelId, MiraError};

use super::writing::{announce, single};
use crate::api::contract::Response;
use crate::api::engine::Engine;
use crate::storage::repositories::local_labels::require_label;

/// Change la couleur ; libellé inconnu : `not_found.local_label`
pub(crate) fn handle_recolor_local_label(
    engine: &Engine,
    label: &LocalLabelId,
    color: u32,
) -> Result<Response, MiraError> {
    let recolored = engine.database.write(|tx| {
        tx.recolor_local_label(label, color)?;
        require_label(tx, label)
    })?;
    announce(engine)?;
    Ok(single(recolored))
}
