//! Appels du domaine « rule » par la façade et lecture de leurs réponses.

use mira_core::api::{Engine, Request, Response, RuleRequest, RuleResponse, RuleView};
use mira_vocabulary::{IdempotencyKey, MiraError, Rule};

/// Exécute une requête du domaine règles
pub async fn rule_call(engine: &Engine, request: RuleRequest) -> Result<Response, MiraError> {
    engine.execute(Request::Rule(request)).await
}

/// Crée une règle sous cette clé et rend sa vue, ou le code d'erreur
pub async fn create(engine: &Engine, rule: Rule, key_text: &str) -> Result<RuleView, String> {
    let request = RuleRequest::CreateRule {
        rule,
        idempotency_key: key(key_text),
    };
    view_of(rule_call(engine, request).await)
}

/// Vue d'une règle extraite d'une réponse, ou le code d'erreur
pub fn view_of(result: Result<Response, MiraError>) -> Result<RuleView, String> {
    match result {
        Ok(Response::Rule(RuleResponse::Rule(view))) => Ok(view),
        Ok(other) => panic!("réponse inattendue : {other:?}"),
        Err(error) => Err(error.code),
    }
}

/// Règles listées par le moteur, dans l'ordre
pub async fn list(engine: &Engine) -> Vec<RuleView> {
    match rule_call(engine, RuleRequest::ListRules).await.unwrap() {
        Response::Rule(RuleResponse::Rules(views)) => views,
        other => panic!("réponse inattendue : {other:?}"),
    }
}

/// Clé d'idempotence de test
pub fn key(raw: &str) -> IdempotencyKey {
    IdempotencyKey(raw.to_string())
}

/// Identifiants des règles listées, dans l'ordre
pub async fn listed_ids(engine: &Engine) -> Vec<String> {
    let views = list(engine).await;
    views
        .iter()
        .map(|view| view.rule.id.as_str().to_string())
        .collect()
}
