//! Exemples d'export et d'import YAML des libellés locaux.

use mira_core::api::{LocalLabelRequest, LocalLabelResponse, Response};
use mira_vocabulary::{
    IdempotencyKey, LabelImportEntry, LabelImportOutcome, LabelImportRefusal, LabelImportReport,
};

use super::Named;

/// Les deux requêtes : export, et import avec remplacement
pub fn requests() -> Vec<LocalLabelRequest> {
    vec![
        LocalLabelRequest::ExportLocalLabelsFile,
        LocalLabelRequest::ImportLocalLabelsFile {
            yaml: "mira_export:\n  kind: local_labels\n  version: 1\nlabels: []\n".to_string(),
            replace_existing: true,
            idempotency_key: IdempotencyKey("key-1".to_string()),
        },
    ]
}

/// Rapport d'exemple : un sort de chaque sorte
fn sample_report() -> LabelImportReport {
    let entry = |position, name: &str, outcome| LabelImportEntry {
        position,
        name: name.to_string(),
        outcome,
    };
    let refused = |reason| LabelImportOutcome::Refused { reason };
    LabelImportReport {
        entries: vec![
            entry(1, "Compta", LabelImportOutcome::Imported),
            entry(2, "Voyages", LabelImportOutcome::Replaced),
            entry(3, "Banque", LabelImportOutcome::AlreadyPresent),
            entry(4, "Rouge", refused(LabelImportRefusal::InvalidColor)),
            entry(5, "compta", refused(LabelImportRefusal::DuplicateInFile)),
        ],
    }
}

/// Les deux réponses : texte exporté et rapport d'import
pub fn responses() -> Vec<Named<Response>> {
    let export = LocalLabelResponse::LabelsFileExport {
        yaml: "mira_export: {}\n".to_string(),
    };
    let import = LocalLabelResponse::LabelsFileImport(sample_report());
    vec![
        (
            "local_label.labels_file_export".to_string(),
            Response::LocalLabel(export),
        ),
        (
            "local_label.labels_file_import".to_string(),
            Response::LocalLabel(import),
        ),
    ]
}
