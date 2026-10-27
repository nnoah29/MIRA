//! Export et import YAML des règles par la façade, et lecture de leurs éléments.

use mira_core::api::{Engine, Response, RuleRequest, RuleResponse};
use mira_core::portability::envelope::{unwrap, wrap};
use mira_vocabulary::{RuleImportOutcome, RuleImportReport};
use serde_json::Value;

use super::calls::{key, rule_call};

/// Texte YAML exporté, ou code d'erreur
pub async fn export_yaml(engine: &Engine) -> Result<String, String> {
    match rule_call(engine, RuleRequest::ExportRules).await {
        Ok(Response::Rule(RuleResponse::RulesExport { yaml })) => Ok(yaml),
        Ok(other) => panic!("réponse inattendue : {other:?}"),
        Err(error) => Err(error.code),
    }
}

/// Règles du fichier exporté, dans l'ordre du fichier
pub async fn exported_items(engine: &Engine) -> Vec<Value> {
    let yaml = export_yaml(engine).await.unwrap();
    unwrap(&yaml, "rules", "rules").unwrap()
}

/// Importe ce texte sous cette clé et rend le rapport, ou le code d'erreur
pub async fn import(
    engine: &Engine,
    yaml: &str,
    replace_existing: bool,
    key_text: &str,
) -> Result<RuleImportReport, String> {
    let request = RuleRequest::ImportRules {
        yaml: yaml.to_string(),
        replace_existing,
        idempotency_key: key(key_text),
    };
    match rule_call(engine, request).await {
        Ok(Response::Rule(RuleResponse::RulesImport(report))) => Ok(report),
        Ok(other) => panic!("réponse inattendue : {other:?}"),
        Err(error) => Err(error.code),
    }
}

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
