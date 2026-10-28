//! Références en clair du fichier vers identifiants locaux.

use serde_json::Value;

use super::from_portable::PortableError;
use super::refs::RuleRefs;

/// Identifiant local d'un champ de référence du fichier
pub(super) fn import_reference(
    key: &str,
    child: &Value,
    refs: &dyn RuleRefs,
) -> Result<Value, PortableError> {
    let text = child.as_str().ok_or(PortableError::Invalid);
    match key {
        "account" => account_id(text?, refs),
        "accounts" => accounts_ids(child, refs),
        "mailbox" => mailbox_id(child, refs),
        "label" => refs
            .label_by_name(text?)
            .map(|id| Value::String(id.as_str().to_string()))
            .ok_or(PortableError::UnknownLabel),
        "template" => refs
            .template_by_name(text?)
            .map(|id| Value::String(id.as_str().to_string()))
            .ok_or(PortableError::UnknownTemplate),
        _ => Err(PortableError::Invalid),
    }
}

/// Identifiant du compte portant cette adresse
fn account_id(address: &str, refs: &dyn RuleRefs) -> Result<Value, PortableError> {
    refs.account_by_address(address)
        .map(|id| Value::String(id.as_str().to_string()))
        .ok_or(PortableError::UnknownAccount)
}

/// Identifiants d'une liste d'adresses de comptes
fn accounts_ids(child: &Value, refs: &dyn RuleRefs) -> Result<Value, PortableError> {
    let items = child.as_array().ok_or(PortableError::Invalid)?;
    let ids = items
        .iter()
        .map(|item| account_id(item.as_str().ok_or(PortableError::Invalid)?, refs))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Value::Array(ids))
}

/// Identifiant de la boîte désignée par compte et nom serveur
fn mailbox_id(child: &Value, refs: &dyn RuleRefs) -> Result<Value, PortableError> {
    let field = |name: &str| {
        child
            .get(name)
            .and_then(Value::as_str)
            .ok_or(PortableError::Invalid)
    };
    let (address, name) = (field("account")?, field("name")?);
    account_id(address, refs)?;
    refs.mailbox_by_ref(address, name)
        .map(|id| Value::String(id.as_str().to_string()))
        .ok_or(PortableError::UnknownMailbox)
}
