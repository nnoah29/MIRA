//! Fichiers d'export de test : construction, copie d'une règle exportée, sorts d'un rapport.

use mira_core::portability::envelope::wrap;
use mira_vocabulary::{RuleImportOutcome, RuleImportReport};
use serde_json::Value;

/// Fichier d'export portant ces règles
pub fn file_of(items: Vec<Value>) -> String {
    wrap("rules", "rules", items).unwrap()
}

/// Sorts du rapport, dans l'ordre du fichier
pub fn outcomes(report: &RuleImportReport) -> Vec<RuleImportOutcome> {
    report
        .entries
        .iter()
        .map(|entry| entry.outcome.clone())
        .collect()
}

/// Copie d'une règle exportée sous un autre identifiant et un autre nom
pub fn variant(item: &Value, id: &str, name: &str) -> Value {
    let mut copy = item.clone();
    copy["id"] = Value::String(id.to_string());
    copy["name"] = Value::String(name.to_string());
    copy
}
