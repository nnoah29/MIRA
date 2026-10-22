//! Catalogue des conditions et actions : dérivé d'une instance de chaque variante, le moteur reste la source de vérité.

mod action_samples;
mod condition_samples;
mod families;

use mira_vocabulary::error::codes::internal;
use mira_vocabulary::{ConditionTest, ErrorCategory, MiraError, RuleAction};
use serde_json::Value;

use crate::api::contract::{CatalogEntry, Response, RuleCatalog, RuleResponse};
use crate::rules::validate::{depends_on_account_mailbox, named_mailbox};

/// Rend toutes les variantes de conditions et d'actions, avec leurs paramètres et leurs besoins
pub(crate) fn handle_list_rule_catalog() -> Result<Response, MiraError> {
    let conditions = condition_samples::condition_samples()
        .iter()
        .map(condition_entry)
        .collect::<Result<Vec<_>, _>>()?;
    let actions = action_samples::action_samples()
        .iter()
        .map(action_entry)
        .collect::<Result<Vec<_>, _>>()?;
    let catalog = RuleCatalog {
        conditions,
        actions,
    };
    Ok(Response::Rule(RuleResponse::Catalog(catalog)))
}

/// Entrée d'une condition : seule « dans la boîte » désigne une boîte
fn condition_entry(test: &ConditionTest) -> Result<CatalogEntry, MiraError> {
    let mut entry = entry_from_json(serde_json::to_value(test))?;
    entry.family = families::condition_family(test).to_string();
    entry.needs_mailbox = matches!(test, ConditionTest::InMailbox { .. });
    Ok(entry)
}

/// Entrée d'une action : boîte nommée, ou boîte du compte exigeant une portée d'un seul compte
fn action_entry(action: &RuleAction) -> Result<CatalogEntry, MiraError> {
    let mut entry = entry_from_json(serde_json::to_value(action))?;
    entry.family = families::action_family(action).to_string();
    entry.needs_mailbox = named_mailbox(action).is_some();
    entry.single_account = depends_on_account_mailbox(action);
    Ok(entry)
}

/// Étiquette `type` et noms des paramètres lus dans la forme JSON de l'instance
fn entry_from_json(json: Result<Value, serde_json::Error>) -> Result<CatalogEntry, MiraError> {
    let invalid = || {
        MiraError::new(
            ErrorCategory::Internal,
            internal::BUG,
            "variante de règle sans étiquette JSON",
        )
    };
    let Ok(Value::Object(fields)) = json else {
        return Err(invalid());
    };
    let kind = fields
        .get("type")
        .and_then(Value::as_str)
        .ok_or_else(invalid)?;
    Ok(CatalogEntry {
        kind: kind.to_string(),
        family: String::new(),
        fields: fields
            .keys()
            .filter(|name| *name != "type")
            .cloned()
            .collect(),
        needs_mailbox: false,
        single_account: false,
    })
}
