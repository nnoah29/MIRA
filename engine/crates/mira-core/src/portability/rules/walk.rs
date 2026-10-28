//! Parcours d'une règle en JSON : seuls les champs de référence sont réécrits.

use serde_json::Value;

const REFERENCE_KEYS: [&str; 5] = ["account", "accounts", "mailbox", "label", "template"];

/// Remplace chaque champ de référence par le résultat de `convert`, sans descendre dans le remplacement
pub(super) fn rewrite<E>(
    value: &mut Value,
    convert: &mut dyn FnMut(&str, &Value) -> Result<Value, E>,
) -> Result<(), E> {
    match value {
        Value::Object(map) => {
            for (key, child) in map.iter_mut() {
                if REFERENCE_KEYS.contains(&key.as_str()) {
                    *child = convert(key, child)?;
                } else {
                    rewrite(child, convert)?;
                }
            }
            Ok(())
        }
        Value::Array(items) => items.iter_mut().try_for_each(|item| rewrite(item, convert)),
        _ => Ok(()),
    }
}
