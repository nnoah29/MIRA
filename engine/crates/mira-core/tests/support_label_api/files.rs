//! Export et import YAML des libellés par la façade, et fichiers de test.

use mira_core::api::{Engine, LocalLabelRequest, LocalLabelResponse};
use mira_core::portability::envelope::{unwrap, wrap};
use mira_vocabulary::{LabelImportOutcome, LabelImportReport, LocalLabel};
use serde_json::Value;

use super::calls::{call, key};

/// Crée un libellé de cette couleur sous cette clé
pub async fn create_colored(engine: &Engine, name: &str, color: u32, key_text: &str) -> LocalLabel {
    let request = LocalLabelRequest::CreateLocalLabel {
        name: name.to_string(),
        color: Some(color),
        idempotency_key: key(key_text),
    };
    match call(engine, request).await.unwrap() {
        LocalLabelResponse::Label(label) => label,
        other => panic!("réponse inattendue : {other:?}"),
    }
}

/// Texte YAML exporté, ou code d'erreur
pub async fn export_yaml(engine: &Engine) -> Result<String, String> {
    match call(engine, LocalLabelRequest::ExportLocalLabelsFile).await? {
        LocalLabelResponse::LabelsFileExport { yaml } => Ok(yaml),
        other => panic!("réponse inattendue : {other:?}"),
    }
}

/// Entrées du fichier exporté, dans l'ordre du fichier
pub async fn exported_items(engine: &Engine) -> Vec<Value> {
    let yaml = export_yaml(engine).await.unwrap();
    unwrap(&yaml, "local_labels", "labels").unwrap()
}

/// Fichier d'export portant ces entrées
pub fn file_of(items: Vec<Value>) -> String {
    wrap("local_labels", "labels", items).unwrap()
}

/// Sorts du rapport, dans l'ordre du fichier
pub fn outcomes(report: &LabelImportReport) -> Vec<LabelImportOutcome> {
    report.entries.iter().map(|e| e.outcome.clone()).collect()
}

/// Importe ce texte sous cette clé et rend le rapport, ou le code d'erreur
pub async fn import_file(
    engine: &Engine,
    yaml: &str,
    replace_existing: bool,
    key_text: &str,
) -> Result<LabelImportReport, String> {
    let request = LocalLabelRequest::ImportLocalLabelsFile {
        yaml: yaml.to_string(),
        replace_existing,
        idempotency_key: key(key_text),
    };
    match call(engine, request).await? {
        LocalLabelResponse::LabelsFileImport(report) => Ok(report),
        other => panic!("réponse inattendue : {other:?}"),
    }
}
