//! Export et import YAML des règles par la façade, et lecture de leurs éléments.

use mira_core::api::{Engine, Response, RuleRequest, RuleResponse};
use mira_core::portability::envelope::unwrap;
use mira_vocabulary::RuleImportReport;
use serde_json::Value;

use super::calls::{key, rule_call};

pub use super::files::{file_of, outcomes, variant};

/// Texte YAML exporté et noms des règles laissées de côté, ou code d'erreur
pub async fn export_response(engine: &Engine) -> Result<(String, Vec<String>), String> {
    match rule_call(engine, RuleRequest::ExportRules).await {
        Ok(Response::Rule(RuleResponse::RulesExport { yaml, skipped })) => Ok((yaml, skipped)),
        Ok(other) => panic!("réponse inattendue : {other:?}"),
        Err(error) => Err(error.code),
    }
}

/// Texte YAML exporté, ou code d'erreur
pub async fn export_yaml(engine: &Engine) -> Result<String, String> {
    export_response(engine).await.map(|(yaml, _)| yaml)
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
