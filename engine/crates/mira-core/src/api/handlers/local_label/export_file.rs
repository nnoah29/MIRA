//! Export des libellés locaux en texte YAML portable : nom et couleur, sans pose.

use mira_vocabulary::MiraError;

use crate::api::contract::{LocalLabelResponse, Response};
use crate::api::engine::Engine;
use crate::portability::envelope::wrap;
use crate::portability::labels::to_portable;

/// Écrit tous les libellés, dans l'ordre de la liste, sous l'enveloppe `local_labels`
pub(crate) fn handle_export_labels_file(engine: &Engine) -> Result<Response, MiraError> {
    let labels = engine.database.read(|tx| tx.list_local_labels())?;
    let items = labels.iter().map(to_portable).collect();
    let yaml = wrap("local_labels", "labels", items)?;
    Ok(Response::LocalLabel(LocalLabelResponse::LabelsFileExport {
        yaml,
    }))
}
