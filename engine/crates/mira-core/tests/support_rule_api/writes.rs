//! Écritures du domaine « rule » par la façade, réponse réduite à la valeur utile ou au code d'erreur.

use mira_core::api::{Engine, Response, RuleRequest, RuleResponse, RuleView};
use mira_vocabulary::{Rule, RuleId, RuleScope};

use super::calls::{create, key, rule_call, view_of};
use super::samples::subject_rule;

/// Crée une règle « tous comptes » par nom, sous la clé égale au nom, et rend leurs identifiants
pub async fn create_named(engine: &Engine, names: &[&str]) -> Vec<RuleId> {
    let mut ids = Vec::new();
    for name in names {
        let rule = subject_rule(name, name, RuleScope::AllAccounts);
        ids.push(create(engine, rule, name).await.unwrap().rule.id);
    }
    ids
}

/// Remplace une règle sous cette clé
pub async fn update(engine: &Engine, rule: Rule, key_text: &str) -> Result<RuleView, String> {
    let request = RuleRequest::UpdateRule {
        rule,
        idempotency_key: key(key_text),
    };
    view_of(rule_call(engine, request).await)
}

/// Active ou désactive une règle sous cette clé
pub async fn set_enabled(
    engine: &Engine,
    rule: &RuleId,
    enabled: bool,
    key_text: &str,
) -> Result<RuleView, String> {
    let request = RuleRequest::SetRuleEnabled {
        rule: rule.clone(),
        enabled,
        idempotency_key: key(key_text),
    };
    view_of(rule_call(engine, request).await)
}

/// Réordonne sous cette clé et rend l'ordre répondu
pub async fn reorder(
    engine: &Engine,
    order: &[RuleId],
    key_text: &str,
) -> Result<Vec<RuleId>, String> {
    let request = RuleRequest::ReorderRules {
        order: order.to_vec(),
        idempotency_key: key(key_text),
    };
    match rule_call(engine, request).await {
        Ok(Response::Rule(RuleResponse::Rules(views))) => {
            Ok(views.into_iter().map(|view| view.rule.id).collect())
        }
        Ok(other) => panic!("réponse inattendue : {other:?}"),
        Err(error) => Err(error.code),
    }
}

/// Supprime une règle sous cette clé
pub async fn delete(engine: &Engine, rule: &RuleId, key_text: &str) -> Result<(), String> {
    let request = RuleRequest::DeleteRule {
        rule: rule.clone(),
        idempotency_key: key(key_text),
    };
    match rule_call(engine, request).await {
        Ok(Response::Done) => Ok(()),
        Ok(other) => panic!("réponse inattendue : {other:?}"),
        Err(error) => Err(error.code),
    }
}
