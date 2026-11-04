//! Export des libellés locaux en texte YAML portable : nom, couleur et réglages, sans pose.

use mira_vocabulary::{LabelKind, LabelRef, MiraError};

use crate::api::contract::{LocalLabelResponse, Response};
use crate::api::engine::Engine;
use crate::portability::envelope::wrap;
use crate::portability::labels::to_portable_with_settings;

/// Écrit tous les libellés avec leurs réglages, dans l'ordre de la liste, sous l'enveloppe `local_labels`
pub(crate) fn handle_export_labels_file(engine: &Engine) -> Result<Response, MiraError> {
    let items = engine.database.read(|tx| {
        let mut items = Vec::new();
        for label in tx.list_local_labels()? {
            let reference = LabelRef {
                kind: LabelKind::Local,
                id: label.id.as_str().to_string(),
            };
            let settings = tx.get_label_settings(&reference)?;
            items.push(to_portable_with_settings(&label, &settings));
        }
        Ok(items)
    })?;
    let yaml = wrap("local_labels", "labels", items)?;
    Ok(Response::LocalLabel(LocalLabelResponse::LabelsFileExport {
        yaml,
    }))
}
