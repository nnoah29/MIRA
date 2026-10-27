//! Forme portable vers règle : références résolues, champs inconnus et identifiant vide refusés.

use mira_vocabulary::Rule;
use serde_json::Value;

use super::import_refs::import_reference;
use super::refs::RuleRefs;
use super::walk::rewrite;

/// Raison pour laquelle une règle du fichier ne peut pas devenir une règle locale
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortableError {
    UnknownAccount,
    UnknownMailbox,
    UnknownLabel,
    UnknownTemplate,
    Invalid,
}

/// Résout les références puis relit la règle ; tout champ inconnu rend la règle invalide
pub fn from_portable(mut value: Value, refs: &dyn RuleRefs) -> Result<Rule, PortableError> {
    let has_id = value
        .get("id")
        .and_then(Value::as_str)
        .is_some_and(|id| !id.trim().is_empty());
    if !has_id {
        return Err(PortableError::Invalid);
    }
    rewrite(&mut value, &mut |key, child| {
        import_reference(key, child, refs)
    })?;
    let rule: Rule = serde_json::from_value(value.clone()).map_err(|_| PortableError::Invalid)?;
    match serde_json::to_value(&rule) {
        Ok(again) if again == value => Ok(rule),
        _ => Err(PortableError::Invalid),
    }
}
