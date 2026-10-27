//! Règle vers sa forme portable : chaque identifiant local devient une référence en clair.

use mira_vocabulary::error::codes::{internal, invalid_input};
use mira_vocabulary::{
    AccountId, ErrorCategory, LocalLabelId, MailboxId, MiraError, Rule, TemplateId,
};
use serde_json::{Value, json};

use super::refs::RuleRefs;
use super::walk::rewrite;

/// Convertit la règle ; une référence dont la cible a disparu fait échouer l'export
pub fn to_portable(rule: &Rule, refs: &dyn RuleRefs) -> Result<Value, MiraError> {
    let mut value = serde_json::to_value(rule).map_err(|_| {
        MiraError::new(
            ErrorCategory::Internal,
            internal::BUG,
            "règle non sérialisable",
        )
    })?;
    rewrite(&mut value, &mut |key, child| {
        export_reference(key, child, refs)
    })?;
    Ok(value)
}

/// Référence en clair d'un champ de référence, ou erreur si la cible n'existe plus
fn export_reference(key: &str, child: &Value, refs: &dyn RuleRefs) -> Result<Value, MiraError> {
    let text = child.as_str();
    let converted = match key {
        "account" => text
            .and_then(|id| refs.account_address(&AccountId::from_raw(id)))
            .map(Value::String),
        "accounts" => export_accounts(child, refs),
        "mailbox" => text
            .and_then(|id| refs.mailbox_ref(&MailboxId::from_raw(id)))
            .map(|(account, name)| json!({ "account": account, "name": name })),
        "label" => text
            .and_then(|id| refs.label_name(&LocalLabelId::from_raw(id)))
            .map(Value::String),
        "template" => text
            .and_then(|id| refs.template_name(&TemplateId::from_raw(id)))
            .map(Value::String),
        _ => None,
    };
    converted.ok_or_else(|| {
        MiraError::new(
            ErrorCategory::InvalidInput,
            invalid_input::PARAMETER,
            "une règle vise un objet qui n'existe plus",
        )
    })
}

/// Adresses d'une liste d'identifiants de comptes
fn export_accounts(child: &Value, refs: &dyn RuleRefs) -> Option<Value> {
    let addresses = child
        .as_array()?
        .iter()
        .map(|item| {
            item.as_str()
                .and_then(|id| refs.account_address(&AccountId::from_raw(id)))
        })
        .collect::<Option<Vec<_>>>()?;
    Some(Value::Array(
        addresses.into_iter().map(Value::String).collect(),
    ))
}
