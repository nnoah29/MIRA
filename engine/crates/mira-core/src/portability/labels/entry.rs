//! Entrée complète d'un libellé local du fichier : libellé (nom, couleur) et réglages facultatifs.

use mira_vocabulary::{LabelSettings, LocalLabel};
use serde_json::Value;

use super::from_portable::{PortableLabel, PortableLabelError, from_portable};
use super::settings::{PortableSettings, take_settings, write_settings};
use super::to_portable::to_portable;

/// Forme portable d'un libellé local avec ses réglages
pub fn to_portable_with_settings(label: &LocalLabel, settings: &LabelSettings) -> Value {
    let mut value = to_portable(label);
    if let Some(map) = value.as_object_mut() {
        write_settings(map, settings);
    }
    value
}

/// Lit une entrée : les réglages sont séparés d'abord, le reste suit la lecture stricte du libellé
pub fn read_portable_entry(
    value: Value,
) -> Result<(PortableLabel, PortableSettings), PortableLabelError> {
    let Value::Object(mut map) = value else {
        return Err(PortableLabelError::Invalid);
    };
    let settings = take_settings(&mut map).ok_or(PortableLabelError::Invalid)?;
    let label = from_portable(Value::Object(map))?;
    Ok((label, settings))
}
