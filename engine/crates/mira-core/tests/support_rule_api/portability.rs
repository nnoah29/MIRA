//! Export et import YAML des règles par la façade, et lecture de leurs éléments.

use mira_core::api::{Engine, Response, RuleRequest, RuleResponse};
use mira_core::portability::envelope::unwrap;
use serde_json::Value;

use super::calls::rule_call;

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
